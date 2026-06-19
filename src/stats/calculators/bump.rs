use super::*;

const BUMP_MAX_SAMPLE_DT: f32 = 0.18;
const BUMP_MAX_CONTACT_GAP: f32 = 35.0;
const BUMP_CONTACT_GAP_SAMPLES: usize = 8;
const BUMP_MIN_CLOSING_SPEED: f32 = 420.0;
const BUMP_MIN_VICTIM_IMPULSE: f32 = 180.0;
const BUMP_MIN_INITIATOR_SLOWDOWN: f32 = 100.0;
const BUMP_MIN_DIRECTIONAL_SCORE: f32 = 650.0;
const BUMP_MIN_SCORE_MARGIN: f32 = 175.0;
const BUMP_REPEAT_FRAME_WINDOW: usize = 10;
const BUMP_FIFTY_FIFTY_SUPPRESSION_WINDOW_SECONDS: f32 = 0.35;

/// A player-on-player bump with attacker/victim and impact context.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub initiator: PlayerId,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    pub initiator_is_team_0: bool,
    pub victim_is_team_0: bool,
    pub is_team_bump: bool,
    pub strength: f32,
    pub confidence: f32,
    pub contact_distance: f32,
    pub closing_speed: f32,
    pub victim_impulse: f32,
    pub initiator_position: [f32; 3],
    pub victim_position: [f32; 3],
}

#[derive(Debug, Clone)]
struct PreviousPlayerSample {
    rigid_body: boxcars::RigidBody,
}

#[derive(Debug, Clone, Copy)]
struct DirectionalBumpCandidate {
    score: f32,
    closing_speed: f32,
    victim_impulse: f32,
    initiator_slowdown: f32,
}

/// Detects player-on-player bumps from player frame state and events.
#[derive(Debug, Clone, Default)]
pub struct BumpCalculator {
    events: EventStream<BumpEvent>,
    previous_players: HashMap<PlayerId, PreviousPlayerSample>,
    last_seen_pair_frame: HashMap<(PlayerId, PlayerId), usize>,
}

impl BumpCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[BumpEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BumpEvent] {
        self.events.new_events()
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.update_with_fifty_fifty_state(
            frame,
            players,
            events,
            &FiftyFiftyState::default(),
            live_play_state,
        )
    }

    pub fn update_with_fifty_fifty_state(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();

        if !live_play_state.is_live_play {
            self.previous_players.clear();
            return Ok(());
        }

        if frame.dt > 0.0 && frame.dt <= BUMP_MAX_SAMPLE_DT {
            self.detect_bumps(frame, players, events, fifty_fifty_state);
        }

        self.previous_players = players
            .players
            .iter()
            .filter_map(|player| {
                Some((
                    player.player_id.clone(),
                    PreviousPlayerSample {
                        rigid_body: player.rigid_body?,
                    },
                ))
            })
            .collect();

        Ok(())
    }

    fn detect_bumps(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        frame_events: &FrameEventsState,
        fifty_fifty_state: &FiftyFiftyState,
    ) {
        let current_players: Vec<_> = players
            .players
            .iter()
            .filter_map(|player| {
                Some((
                    player,
                    player.rigid_body.as_ref()?,
                    self.previous_players.get(&player.player_id)?.rigid_body,
                ))
            })
            .collect();

        for left_index in 0..current_players.len() {
            for right_index in (left_index + 1)..current_players.len() {
                let (left, left_body, previous_left_body) = current_players[left_index];
                let (right, right_body, previous_right_body) = current_players[right_index];

                if self.is_recent_demo_pair(frame_events, &left.player_id, &right.player_id) {
                    continue;
                }

                if Self::is_recent_fifty_fifty_pair(
                    frame,
                    fifty_fifty_state,
                    &left.player_id,
                    &right.player_id,
                ) {
                    continue;
                }

                let Some(event) = Self::evaluate_pair(
                    frame,
                    left,
                    left_body,
                    &previous_left_body,
                    right,
                    right_body,
                    &previous_right_body,
                ) else {
                    continue;
                };

                if self.should_count_bump(&event.initiator, &event.victim, frame.frame_number) {
                    self.record_bump(event);
                }
            }
        }
    }

    fn evaluate_pair(
        frame: &FrameInfo,
        left: &PlayerSample,
        left_body: &boxcars::RigidBody,
        previous_left_body: &boxcars::RigidBody,
        right: &PlayerSample,
        right_body: &boxcars::RigidBody,
        previous_right_body: &boxcars::RigidBody,
    ) -> Option<BumpEvent> {
        let left_previous_position = vec_to_glam(&previous_left_body.location);
        let right_previous_position = vec_to_glam(&previous_right_body.location);
        let left_position = vec_to_glam(&left_body.location);
        let right_position = vec_to_glam(&right_body.location);

        let contact_distance = swept_car_hitbox_contact_gap(
            previous_left_body,
            left_body,
            left.hitbox,
            previous_right_body,
            right_body,
            right.hitbox,
        )?;
        if contact_distance > BUMP_MAX_CONTACT_GAP {
            return None;
        }

        let normal_left_to_right = contact_normal(
            left_previous_position,
            left_position,
            right_previous_position,
            right_position,
        )?;
        let left_to_right = directional_candidate(
            previous_left_body,
            left_body,
            previous_right_body,
            right_body,
            normal_left_to_right,
        )?;
        let right_to_left = directional_candidate(
            previous_right_body,
            right_body,
            previous_left_body,
            left_body,
            -normal_left_to_right,
        )?;

        let (initiator, victim, initiator_body, victim_body, candidate, reverse_score) =
            if left_to_right.score >= right_to_left.score {
                (
                    left,
                    right,
                    left_body,
                    right_body,
                    left_to_right,
                    right_to_left.score,
                )
            } else {
                (
                    right,
                    left,
                    right_body,
                    left_body,
                    right_to_left,
                    left_to_right.score,
                )
            };

        if candidate.score < BUMP_MIN_DIRECTIONAL_SCORE
            || candidate.score - reverse_score < BUMP_MIN_SCORE_MARGIN
            || candidate.closing_speed < BUMP_MIN_CLOSING_SPEED
            || candidate.victim_impulse < BUMP_MIN_VICTIM_IMPULSE
            || candidate.initiator_slowdown < BUMP_MIN_INITIATOR_SLOWDOWN
        {
            return None;
        }

        let distance_factor = (1.0 - (contact_distance / BUMP_MAX_CONTACT_GAP)).clamp(0.0, 1.0);
        let score_factor = ((candidate.score - BUMP_MIN_DIRECTIONAL_SCORE) / 900.0).clamp(0.0, 1.0);
        let margin_factor =
            ((candidate.score - reverse_score - BUMP_MIN_SCORE_MARGIN) / 500.0).clamp(0.0, 1.0);
        let confidence = (0.35 + 0.3 * distance_factor + 0.25 * score_factor + 0.1 * margin_factor)
            .clamp(0.0, 1.0);

        Some(BumpEvent {
            time: frame.time,
            frame: frame.frame_number,
            initiator: initiator.player_id.clone(),
            victim: victim.player_id.clone(),
            initiator_is_team_0: initiator.is_team_0,
            victim_is_team_0: victim.is_team_0,
            is_team_bump: initiator.is_team_0 == victim.is_team_0,
            strength: candidate.score,
            confidence,
            contact_distance,
            closing_speed: candidate.closing_speed,
            victim_impulse: candidate.victim_impulse,
            initiator_position: vec3_to_array(vec_to_glam(&initiator_body.location)),
            victim_position: vec3_to_array(vec_to_glam(&victim_body.location)),
        })
    }

    fn is_recent_demo_pair(
        &self,
        frame_events: &FrameEventsState,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        frame_events.demo_events.iter().any(|demo| {
            (&demo.attacker == left && &demo.victim == right)
                || (&demo.attacker == right && &demo.victim == left)
        }) || frame_events.active_demos.iter().any(|demo| {
            (&demo.attacker == left && &demo.victim == right)
                || (&demo.attacker == right && &demo.victim == left)
        })
    }

    fn is_recent_fifty_fifty_pair(
        frame: &FrameInfo,
        fifty_fifty_state: &FiftyFiftyState,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        if fifty_fifty_state
            .active_event
            .as_ref()
            .is_some_and(|event| Self::active_fifty_fifty_matches_pair(event, left, right))
        {
            return true;
        }

        fifty_fifty_state
            .resolved_events
            .iter()
            .any(|event| Self::resolved_fifty_fifty_matches_pair(event, left, right))
            || fifty_fifty_state
                .last_resolved_event
                .as_ref()
                .is_some_and(|event| {
                    frame.time - event.resolve_time <= BUMP_FIFTY_FIFTY_SUPPRESSION_WINDOW_SECONDS
                        && Self::resolved_fifty_fifty_matches_pair(event, left, right)
                })
    }

    fn active_fifty_fifty_matches_pair(
        event: &ActiveFiftyFifty,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        Self::optional_player_pair_matches(
            event.team_zero_player.as_ref(),
            event.team_one_player.as_ref(),
            left,
            right,
        )
    }

    fn resolved_fifty_fifty_matches_pair(
        event: &FiftyFiftyEvent,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        Self::optional_player_pair_matches(
            event.team_zero_player.as_ref(),
            event.team_one_player.as_ref(),
            left,
            right,
        )
    }

    fn optional_player_pair_matches(
        team_zero_player: Option<&PlayerId>,
        team_one_player: Option<&PlayerId>,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        matches!(
            (team_zero_player, team_one_player),
            (Some(team_zero_player), Some(team_one_player))
                if (team_zero_player == left && team_one_player == right)
                    || (team_zero_player == right && team_one_player == left)
        )
    }

    fn should_count_bump(
        &mut self,
        initiator: &PlayerId,
        victim: &PlayerId,
        frame_number: usize,
    ) -> bool {
        let key = (initiator.clone(), victim.clone());
        let already_counted = self
            .last_seen_pair_frame
            .get(&key)
            .map(|previous_frame| {
                frame_number.saturating_sub(*previous_frame) <= BUMP_REPEAT_FRAME_WINDOW
            })
            .unwrap_or(false);
        self.last_seen_pair_frame.insert(key, frame_number);
        !already_counted
    }

    fn record_bump(&mut self, event: BumpEvent) {
        self.events.push(event);
    }
}

fn vec3_to_array(v: glam::Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn swept_car_hitbox_contact_gap(
    left_previous: &boxcars::RigidBody,
    left_current: &boxcars::RigidBody,
    left_hitbox: CarHitbox,
    right_previous: &boxcars::RigidBody,
    right_current: &boxcars::RigidBody,
    right_hitbox: CarHitbox,
) -> Option<f32> {
    let mut closest_gap =
        car_hitbox_pair_contact_gap(left_current, left_hitbox, right_current, right_hitbox)?;

    for sample_index in 0..=BUMP_CONTACT_GAP_SAMPLES {
        let sample_fraction = sample_index as f32 / BUMP_CONTACT_GAP_SAMPLES as f32;
        let left_sample = interpolate_rigid_body(left_previous, left_current, sample_fraction);
        let right_sample = interpolate_rigid_body(right_previous, right_current, sample_fraction);
        let sample_gap =
            car_hitbox_pair_contact_gap(&left_sample, left_hitbox, &right_sample, right_hitbox)?;
        closest_gap = closest_gap.min(sample_gap);
    }

    Some(closest_gap)
}

fn interpolate_rigid_body(
    previous: &boxcars::RigidBody,
    current: &boxcars::RigidBody,
    fraction: f32,
) -> boxcars::RigidBody {
    let fraction = fraction.clamp(0.0, 1.0);
    let previous_position = vec_to_glam(&previous.location);
    let current_position = vec_to_glam(&current.location);
    let previous_rotation = quat_to_glam(&previous.rotation);
    let current_rotation = quat_to_glam(&current.rotation);

    let mut interpolated = *current;
    interpolated.location = glam_to_vec(&previous_position.lerp(current_position, fraction));
    interpolated.rotation = glam_to_quat(&previous_rotation.slerp(current_rotation, fraction));
    interpolated
}

fn contact_normal(
    left_previous: glam::Vec3,
    left_current: glam::Vec3,
    right_previous: glam::Vec3,
    right_current: glam::Vec3,
) -> Option<glam::Vec3> {
    let relative_current = right_current - left_current;
    let current_horizontal = glam::Vec3::new(relative_current.x, relative_current.y, 0.0);
    if current_horizontal.length_squared() > 1.0 {
        return Some(current_horizontal.normalize());
    }

    let relative_previous = right_previous - left_previous;
    let previous_horizontal = glam::Vec3::new(relative_previous.x, relative_previous.y, 0.0);
    (previous_horizontal.length_squared() > 1.0).then(|| previous_horizontal.normalize())
}

fn directional_candidate(
    initiator_previous: &boxcars::RigidBody,
    initiator_current: &boxcars::RigidBody,
    victim_previous: &boxcars::RigidBody,
    victim_current: &boxcars::RigidBody,
    normal: glam::Vec3,
) -> Option<DirectionalBumpCandidate> {
    let initiator_previous_velocity = rigid_body_velocity(initiator_previous);
    let initiator_current_velocity = rigid_body_velocity(initiator_current);
    let victim_previous_velocity = rigid_body_velocity(victim_previous);
    let victim_current_velocity = rigid_body_velocity(victim_current);

    let closing_speed = (initiator_previous_velocity - victim_previous_velocity).dot(normal);
    let victim_impulse = (victim_current_velocity - victim_previous_velocity).dot(normal);
    let initiator_slowdown = (initiator_previous_velocity - initiator_current_velocity).dot(normal);
    let speed_advantage =
        initiator_previous_velocity.dot(normal) - victim_previous_velocity.dot(normal);
    let forward_alignment = (quat_to_glam(&initiator_previous.rotation) * glam::Vec3::X)
        .dot(normal)
        .max(0.0);

    if !closing_speed.is_finite() || !victim_impulse.is_finite() {
        return None;
    }

    let score = closing_speed
        + 1.35 * victim_impulse.max(0.0)
        + 0.35 * initiator_slowdown.max(0.0)
        + 220.0 * forward_alignment
        + 0.15 * speed_advantage.max(0.0);

    Some(DirectionalBumpCandidate {
        score,
        closing_speed,
        victim_impulse,
        initiator_slowdown,
    })
}

fn rigid_body_velocity(rigid_body: &boxcars::RigidBody) -> glam::Vec3 {
    rigid_body
        .linear_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO)
}

#[cfg(test)]
#[path = "bump_tests.rs"]
mod tests;
