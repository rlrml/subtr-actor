use super::*;

const BUMP_MAX_SAMPLE_DT: f32 = 0.18;
const BUMP_MAX_CONTACT_DISTANCE: f32 = 230.0;
const BUMP_MAX_VERTICAL_GAP: f32 = 190.0;
const BUMP_MIN_CLOSING_SPEED: f32 = 420.0;
const BUMP_MIN_VICTIM_IMPULSE: f32 = 90.0;
const BUMP_MIN_DIRECTIONAL_SCORE: f32 = 650.0;
const BUMP_MIN_SCORE_MARGIN: f32 = 175.0;
const BUMP_REPEAT_FRAME_WINDOW: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub initiator: PlayerId,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpPlayerStats {
    pub bumps_inflicted: u32,
    pub bumps_taken: u32,
    pub team_bumps_inflicted: u32,
    pub team_bumps_taken: u32,
    pub last_bump_time: Option<f32>,
    pub last_bump_frame: Option<usize>,
    pub last_bump_strength: Option<f32>,
    pub max_bump_strength: f32,
    pub cumulative_bump_strength: f32,
}

impl BumpPlayerStats {
    pub fn average_bump_strength(&self) -> f32 {
        if self.bumps_inflicted == 0 {
            0.0
        } else {
            self.cumulative_bump_strength / self.bumps_inflicted as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpTeamStats {
    pub bumps_inflicted: u32,
    pub team_bumps_inflicted: u32,
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
}

#[derive(Debug, Clone, Default)]
pub struct BumpCalculator {
    player_stats: HashMap<PlayerId, BumpPlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    team_zero_stats: BumpTeamStats,
    team_one_stats: BumpTeamStats,
    events: Vec<BumpEvent>,
    previous_players: HashMap<PlayerId, PreviousPlayerSample>,
    last_seen_pair_frame: HashMap<(PlayerId, PlayerId), usize>,
}

impl BumpCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BumpPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BumpTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BumpTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[BumpEvent] {
        &self.events
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
        }

        if !live_play {
            self.previous_players.clear();
            return Ok(());
        }

        if frame.dt > 0.0 && frame.dt <= BUMP_MAX_SAMPLE_DT {
            self.detect_bumps(frame, players, events);
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

        let contact_distance = swept_horizontal_distance(
            left_previous_position,
            left_position,
            right_previous_position,
            right_position,
        );
        if contact_distance > BUMP_MAX_CONTACT_DISTANCE {
            return None;
        }

        let vertical_gap = (left_position.z - right_position.z)
            .abs()
            .min((left_previous_position.z - right_previous_position.z).abs());
        if vertical_gap > BUMP_MAX_VERTICAL_GAP {
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
        {
            return None;
        }

        let distance_factor =
            (1.0 - (contact_distance / BUMP_MAX_CONTACT_DISTANCE)).clamp(0.0, 1.0);
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
        let initiator_stats = self
            .player_stats
            .entry(event.initiator.clone())
            .or_default();
        initiator_stats.bumps_inflicted += 1;
        if event.is_team_bump {
            initiator_stats.team_bumps_inflicted += 1;
        }
        initiator_stats.last_bump_time = Some(event.time);
        initiator_stats.last_bump_frame = Some(event.frame);
        initiator_stats.last_bump_strength = Some(event.strength);
        initiator_stats.max_bump_strength = initiator_stats.max_bump_strength.max(event.strength);
        initiator_stats.cumulative_bump_strength += event.strength;

        let victim_stats = self.player_stats.entry(event.victim.clone()).or_default();
        victim_stats.bumps_taken += 1;
        if event.is_team_bump {
            victim_stats.team_bumps_taken += 1;
        }

        match event.initiator_is_team_0 {
            true => {
                self.team_zero_stats.bumps_inflicted += 1;
                if event.is_team_bump {
                    self.team_zero_stats.team_bumps_inflicted += 1;
                }
            }
            false => {
                self.team_one_stats.bumps_inflicted += 1;
                if event.is_team_bump {
                    self.team_one_stats.team_bumps_inflicted += 1;
                }
            }
        }

        self.events.push(event);
    }
}

fn vec3_to_array(v: glam::Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn horizontal(v: glam::Vec3) -> glam::Vec2 {
    glam::Vec2::new(v.x, v.y)
}

fn swept_horizontal_distance(
    left_previous: glam::Vec3,
    left_current: glam::Vec3,
    right_previous: glam::Vec3,
    right_current: glam::Vec3,
) -> f32 {
    let relative_start = horizontal(left_previous - right_previous);
    let relative_delta =
        horizontal((left_current - left_previous) - (right_current - right_previous));
    let closest_t = if relative_delta.length_squared() > f32::EPSILON {
        (-relative_start.dot(relative_delta) / relative_delta.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (relative_start + relative_delta * closest_t).length()
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
