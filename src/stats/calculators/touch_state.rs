use super::*;

/// Shared per-frame raw-touch state.
#[derive(Debug, Clone, Default)]
pub struct TouchState {
    pub touch_events: Vec<TouchEvent>,
    pub last_touch: Option<TouchEvent>,
    pub last_touch_player: Option<PlayerId>,
    pub last_touch_team_is_team_0: Option<bool>,
}

impl TouchState {
    pub fn primary_touch_event(&self) -> Option<&TouchEvent> {
        primary_touch_event(&self.touch_events)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TouchCooldownKey {
    Player(PlayerId),
    Team(bool),
}

/// Last rate-limit-accepted touch for a cooldown key: its time plus whether it
/// carried dodge evidence, so a dodge-powered touch can bypass the rate limit
/// imposed by an earlier passive contact.
#[derive(Debug, Clone, Copy)]
struct LastCooldownTouch {
    time: f32,
    dodge_contact: bool,
}

const TOUCH_SCORING: TouchCandidateScoring = TouchCandidateScoring::DEFAULT;
const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;
const CONTESTED_TOUCH_WINDOW_FRAMES: usize = 1;
const BALL_GRAVITY_Z: f32 = -650.0;

/// Maximum contact gap (uu) at which a replay-reported team touch marker may be
/// attributed to that team's closest player using current-frame geometry alone.
///
/// A `BallHitTeamNum` marker is authoritative that the team touched the ball, but
/// on a contested 50/50 the losing challenger frequently contacts the ball a frame
/// before any detectable trajectory deviation, so it never enters the candidate
/// cache and the marker would otherwise be dropped. When the cache cannot resolve
/// the marker, the closest same-team car sitting on the ball is the touch. Kept
/// within the system's relaxed contact boundary so every emitted touch still
/// reflects a genuine hitbox contact.
const MARKER_CONTACT_ATTRIBUTION_MAX_GAP: f32 = TOUCH_SCORING.relaxed_contact_gap_threshold;

/// Maximum contact gap (uu) for a car to be credited a *geometric contested touch*:
/// a car that is physically on the ball but is never accepted as a deviation
/// candidate because its tight contact lands a frame off the ball's measurable
/// trajectory deviation (the other car in the 50/50 already redirected the ball).
const GEOMETRIC_CONTEST_MAX_GAP: f32 = 5.0;

/// How recently (frames / seconds) an opposing-team touch must have been confirmed
/// for a car now sitting on the ball to be credited a geometric contested touch.
const GEOMETRIC_CONTEST_WINDOW_FRAMES: usize = 4;
const GEOMETRIC_CONTEST_WINDOW_SECONDS: f32 = 0.2;

fn accepted_contact_gap(closest_contact_gap: f32, ball_deviation: BallTrajectoryDeviation) -> bool {
    TOUCH_SCORING.accepts_contact_gap(
        closest_contact_gap,
        ball_deviation.position_deviation,
        ball_deviation.velocity_deviation,
    )
}

fn touch_candidate_score(closest_contact_gap: f32, dodge_contact: bool) -> f32 {
    TOUCH_SCORING.score_contact_gap(closest_contact_gap, dodge_contact)
}

fn touch_event_score(event: &TouchEvent) -> f32 {
    touch_candidate_score(
        event.closest_approach_distance.unwrap_or(f32::INFINITY),
        event.dodge_contact,
    )
}

fn player_sort_key(player: &PlayerId) -> (u8, u64, String) {
    match player {
        boxcars::RemoteId::PlayStation(id) => (0, id.online_id, id.name.clone()),
        boxcars::RemoteId::PsyNet(id) => (1, id.online_id, format!("{:?}", id.unknown1)),
        boxcars::RemoteId::SplitScreen(id) => (2, u64::from(*id), String::new()),
        boxcars::RemoteId::Steam(id) => (3, *id, String::new()),
        boxcars::RemoteId::Switch(id) => (4, id.online_id, format!("{:?}", id.unknown1)),
        boxcars::RemoteId::Xbox(id) => (5, *id, String::new()),
        boxcars::RemoteId::QQ(id) => (6, *id, String::new()),
        boxcars::RemoteId::Epic(id) => (7, 0, id.clone()),
    }
}

fn touch_event_player_sort_key(event: &TouchEvent) -> Option<(u8, u64, String)> {
    event.player.as_ref().map(player_sort_key)
}

pub(crate) fn touch_event_ordering(left: &TouchEvent, right: &TouchEvent) -> std::cmp::Ordering {
    touch_event_score(left)
        .total_cmp(&touch_event_score(right))
        .then_with(|| {
            left.closest_approach_distance
                .unwrap_or(f32::INFINITY)
                .total_cmp(&right.closest_approach_distance.unwrap_or(f32::INFINITY))
        })
        .then_with(|| right.dodge_contact.cmp(&left.dodge_contact))
        .then_with(|| right.frame.cmp(&left.frame))
        .then_with(|| right.time.total_cmp(&left.time))
        .then_with(|| right.player.is_some().cmp(&left.player.is_some()))
        .then_with(|| left.team_is_team_0.cmp(&right.team_is_team_0))
        .then_with(|| touch_event_player_sort_key(left).cmp(&touch_event_player_sort_key(right)))
}

fn touch_event_chronological_ordering(left: &TouchEvent, right: &TouchEvent) -> std::cmp::Ordering {
    TouchEvent::timestamp_ordering(left, right).then_with(|| touch_event_ordering(left, right))
}

fn primary_touch_event(touch_events: &[TouchEvent]) -> Option<&TouchEvent> {
    let latest_touch = touch_events
        .iter()
        .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))?;
    touch_events
        .iter()
        .filter(|event| TouchEvent::timestamp_ordering(event, latest_touch).is_eq())
        .min_by(|left, right| touch_event_ordering(left, right))
}

#[derive(Debug, Clone, Copy, Default)]
struct TouchEventContactFields {
    local_ball_position: Option<[f32; 3]>,
    local_hitbox_point: Option<[f32; 3]>,
    world_hitbox_point: Option<[f32; 3]>,
}

fn touch_event_contact_fields(
    ball_position: glam::Vec3,
    player_body: &boxcars::RigidBody,
    hitbox: CarHitbox,
) -> TouchEventContactFields {
    let Some(estimate) = car_hitbox_contact_estimate(ball_position, player_body, hitbox) else {
        return TouchEventContactFields::default();
    };

    let car_position = vec_to_glam(&player_body.location);
    let car_rotation = quat_to_glam(&player_body.rotation);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let world_hitbox_point = car_position
        + car_rotation * (hitbox_center + hitbox_rotation * estimate.local_contact_point);

    TouchEventContactFields {
        local_ball_position: Some(estimate.local_ball_position.to_array()),
        local_hitbox_point: Some(estimate.local_contact_point.to_array()),
        world_hitbox_point: Some(world_hitbox_point.to_array()),
    }
}

#[derive(Debug, Clone)]
struct RecentTeamTouch {
    frame: usize,
    time: f32,
    team_is_team_0: bool,
}

/// Detects raw ball touches per frame from ball/player state and frame events.
#[derive(Debug, Clone, Default)]
pub struct TouchStateCalculator {
    previous_ball_rigid_body: Option<(boxcars::RigidBody, f32)>,
    current_last_touch: Option<TouchEvent>,
    recent_touch_candidates: HashMap<PlayerId, TouchEvent>,
    last_touch_times: HashMap<TouchCooldownKey, LastCooldownTouch>,
    recent_team_touches: Vec<RecentTeamTouch>,
    next_touch_id: u64,
}

impl TouchStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    fn prune_recent_touch_candidates(&mut self, current_frame: usize) {
        self.recent_touch_candidates.retain(|_, candidate| {
            current_frame.saturating_sub(candidate.frame) <= TOUCH_CANDIDATE_WINDOW_FRAMES
        });
    }

    fn ball_trajectory_deviation(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
    ) -> Option<BallTrajectoryDeviation> {
        let current_ball = ball.sample()?;
        let (previous_ball, previous_time) = &self.previous_ball_rigid_body?;
        ball_trajectory_deviation_with_gravity(
            previous_ball,
            *previous_time,
            &current_ball.rigid_body,
            frame.time,
            BALL_GRAVITY_Z,
        )
    }

    fn proximity_touch_candidates(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        ball_deviation: BallTrajectoryDeviation,
    ) -> Vec<TouchEvent> {
        let Some(ball) = ball.sample() else {
            return Vec::new();
        };

        let mut candidates = players
            .players
            .iter()
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let (closest_contact_gap, _current_contact_gap) =
                    touch_candidate_contact_gap_rank_with_hitbox(
                        &ball.rigid_body,
                        rigid_body,
                        player.hitbox,
                    )?;
                if !accepted_contact_gap(closest_contact_gap, ball_deviation) {
                    return None;
                }
                let contact_fields =
                    touch_event_contact_fields(ball.position(), rigid_body, player.hitbox);

                Some(TouchEvent {
                    touch_id: None,
                    time: frame.time,
                    frame: frame.frame_number,
                    team_is_team_0: player.is_team_0,
                    player: Some(player.player_id.clone()),
                    player_position: Some(rigid_body.location),
                    closest_approach_distance: Some(closest_contact_gap),
                    contact_local_ball_position: contact_fields.local_ball_position,
                    contact_local_hitbox_point: contact_fields.local_hitbox_point,
                    contact_world_hitbox_point: contact_fields.world_hitbox_point,
                    dodge_contact: player.dodge_active,
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(touch_event_ordering);
        candidates
    }

    fn candidate_touch_events(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        ball_deviation: BallTrajectoryDeviation,
    ) -> Vec<TouchEvent> {
        let candidates = self.proximity_touch_candidates(frame, ball, players, ball_deviation);
        let Some(primary) = candidates.first() else {
            return Vec::new();
        };
        let primary_score = touch_candidate_score(
            primary.closest_approach_distance.unwrap_or(f32::INFINITY),
            primary.dodge_contact,
        );
        candidates
            .into_iter()
            .filter(|candidate| {
                let score = touch_candidate_score(
                    candidate.closest_approach_distance.unwrap_or(f32::INFINITY),
                    candidate.dodge_contact,
                );
                score <= primary_score + TOUCH_SCORING.simultaneous_touch_score_margin
            })
            .collect()
    }

    fn update_recent_touch_candidates(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        let Some(ball_deviation) = self.ball_trajectory_deviation(frame, ball) else {
            return;
        };

        for candidate in self.proximity_touch_candidates(frame, ball, players, ball_deviation) {
            let Some(player_id) = candidate.player.clone() else {
                continue;
            };

            if self
                .recent_touch_candidates
                .get(&player_id)
                .is_none_or(|previous| touch_event_ordering(&candidate, previous).is_lt())
            {
                self.recent_touch_candidates.insert(player_id, candidate);
            }
        }
    }

    fn candidate_for_player(&self, player_id: &PlayerId) -> Option<TouchEvent> {
        self.recent_touch_candidates.get(player_id).cloned()
    }

    fn best_candidate_for_team(&self, team_is_team_0: bool) -> Option<TouchEvent> {
        self.recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 == team_is_team_0)
            .min_by(|left, right| touch_event_ordering(left, right))
            .cloned()
    }

    /// Finds the closest same-team car to the ball at the current frame, used to
    /// attribute a replay touch marker that the candidate cache cannot resolve.
    fn current_frame_touch_candidate_for_team(
        &self,
        event: &TouchEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<TouchEvent> {
        let ball = ball.sample()?;

        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == event.team_is_team_0)
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let (closest_contact_gap, _current_contact_gap) =
                    touch_candidate_contact_gap_rank_with_hitbox(
                        &ball.rigid_body,
                        rigid_body,
                        player.hitbox,
                    )?;
                Some((closest_contact_gap, player, rigid_body))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .filter(|(closest_contact_gap, _, _)| {
                *closest_contact_gap <= MARKER_CONTACT_ATTRIBUTION_MAX_GAP
            })
            .map(|(closest_contact_gap, player, rigid_body)| {
                let contact_fields =
                    touch_event_contact_fields(ball.position(), rigid_body, player.hitbox);
                TouchEvent {
                    touch_id: None,
                    time: event.time,
                    frame: event.frame,
                    team_is_team_0: event.team_is_team_0,
                    player: Some(player.player_id.clone()),
                    player_position: Some(rigid_body.location),
                    closest_approach_distance: Some(closest_contact_gap),
                    contact_local_ball_position: contact_fields.local_ball_position,
                    contact_local_hitbox_point: contact_fields.local_hitbox_point,
                    contact_world_hitbox_point: contact_fields.world_hitbox_point,
                    dodge_contact: player.dodge_active,
                }
            })
    }

    /// Finds a concrete current-frame contact for a dodge refresh when no
    /// trajectory-deviation candidate was cached. Flip-reset underside contacts
    /// can refresh the dodge without producing a replay `BallHitTeamNum` marker
    /// or enough ball deviation to enter the candidate cache.
    fn current_frame_touch_candidate_for_dodge_refresh(
        &self,
        dodge_refresh: &DodgeRefreshedEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<TouchEvent> {
        self.previous_ball_rigid_body?;
        let ball = ball.sample()?;
        let player = players
            .players
            .iter()
            .find(|player| player.player_id == dodge_refresh.player)?;
        let rigid_body = player.rigid_body.as_ref()?;
        let (closest_contact_gap, _current_contact_gap) =
            touch_candidate_contact_gap_rank_with_hitbox(
                &ball.rigid_body,
                rigid_body,
                player.hitbox,
            )?;
        if closest_contact_gap > MARKER_CONTACT_ATTRIBUTION_MAX_GAP {
            return None;
        }
        let contact_fields = touch_event_contact_fields(ball.position(), rigid_body, player.hitbox);
        Some(TouchEvent {
            touch_id: None,
            time: dodge_refresh.time,
            frame: dodge_refresh.frame,
            team_is_team_0: dodge_refresh.is_team_0,
            player: Some(dodge_refresh.player.clone()),
            player_position: Some(rigid_body.location),
            closest_approach_distance: Some(closest_contact_gap),
            contact_local_ball_position: contact_fields.local_ball_position,
            contact_local_hitbox_point: contact_fields.local_hitbox_point,
            contact_world_hitbox_point: contact_fields.world_hitbox_point,
            dodge_contact: player.dodge_active,
        })
    }

    /// Attributes a team-only replay touch marker to a concrete player.
    ///
    /// Prefers whichever of the recent candidate cache or the current frame's
    /// geometry puts a car closest to the ball. The cache covers the normal case
    /// where a trajectory deviation already identified the toucher; the
    /// current-frame fallback recovers contested 50/50 touches where the
    /// challenger contacted the ball before any deviation was detectable (so it
    /// never cached) yet is still sitting on the ball at the marker frame.
    fn enrich_team_touch_event_from_recent_cache(
        &self,
        event: &TouchEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<TouchEvent> {
        let gap_of =
            |candidate: &TouchEvent| candidate.closest_approach_distance.unwrap_or(f32::INFINITY);

        // Current-frame geometry is only trustworthy during continuous live play,
        // where a previous ball frame establishes a real trajectory. Without that
        // baseline (e.g. the very first frame after a reset) a car merely parked on
        // a stationary ball must not be credited from a bare team marker.
        let current_frame_candidate = if self.previous_ball_rigid_body.is_some() {
            self.current_frame_touch_candidate_for_team(event, ball, players)
        } else {
            None
        };

        let candidate = match (
            self.best_candidate_for_team(event.team_is_team_0),
            current_frame_candidate,
        ) {
            (Some(cache_candidate), Some(current_candidate)) => {
                if gap_of(&current_candidate) < gap_of(&cache_candidate) {
                    current_candidate
                } else {
                    cache_candidate
                }
            }
            (Some(cache_candidate), None) => cache_candidate,
            (None, Some(current_candidate)) => current_candidate,
            (None, None) => return None,
        };

        Some(TouchEvent {
            touch_id: None,
            time: event.time,
            frame: event.frame,
            team_is_team_0: event.team_is_team_0,
            player: event.player.clone().or(candidate.player),
            player_position: event.player_position.or(candidate.player_position),
            closest_approach_distance: event
                .closest_approach_distance
                .or(candidate.closest_approach_distance),
            contact_local_ball_position: event
                .contact_local_ball_position
                .or(candidate.contact_local_ball_position),
            contact_local_hitbox_point: event
                .contact_local_hitbox_point
                .or(candidate.contact_local_hitbox_point),
            contact_world_hitbox_point: event
                .contact_world_hitbox_point
                .or(candidate.contact_world_hitbox_point),
            dodge_contact: event.dodge_contact || candidate.dodge_contact,
        })
    }

    fn enrich_explicit_touch_event_from_current_frame(
        &self,
        event: &TouchEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> TouchEvent {
        let Some(player_id) = event.player.as_ref() else {
            return event.clone();
        };
        let Some(player) = players
            .players
            .iter()
            .find(|sample| &sample.player_id == player_id)
        else {
            return event.clone();
        };
        let rigid_body = player.rigid_body.as_ref();
        let closest_contact_gap = ball
            .sample()
            .zip(rigid_body)
            .and_then(|(ball, rigid_body)| {
                touch_candidate_contact_gap_rank_with_hitbox(
                    &ball.rigid_body,
                    rigid_body,
                    player.hitbox,
                )
            })
            .map(|(closest_contact_gap, _current_contact_gap)| closest_contact_gap);
        let contact_fields = ball
            .position()
            .zip(rigid_body)
            .map(|(ball_position, rigid_body)| {
                touch_event_contact_fields(ball_position, rigid_body, player.hitbox)
            })
            .unwrap_or_default();

        TouchEvent {
            team_is_team_0: player.is_team_0,
            player_position: event
                .player_position
                .or_else(|| rigid_body.map(|rigid_body| rigid_body.location)),
            closest_approach_distance: event.closest_approach_distance.or(closest_contact_gap),
            contact_local_ball_position: event
                .contact_local_ball_position
                .or(contact_fields.local_ball_position),
            contact_local_hitbox_point: event
                .contact_local_hitbox_point
                .or(contact_fields.local_hitbox_point),
            contact_world_hitbox_point: event
                .contact_world_hitbox_point
                .or(contact_fields.world_hitbox_point),
            dodge_contact: event.dodge_contact || player.dodge_active,
            ..event.clone()
        }
    }

    fn touch_event_from_dodge_refresh(
        dodge_refresh: &DodgeRefreshedEvent,
        candidate: TouchEvent,
    ) -> TouchEvent {
        TouchEvent {
            touch_id: None,
            time: dodge_refresh.time,
            frame: dodge_refresh.frame,
            team_is_team_0: dodge_refresh.is_team_0,
            player: Some(dodge_refresh.player.clone()),
            player_position: dodge_refresh
                .player_position
                .map(|position| glam_to_vec(&glam::Vec3::from_array(position)))
                .or(candidate.player_position),
            closest_approach_distance: candidate.closest_approach_distance,
            contact_local_ball_position: candidate.contact_local_ball_position,
            contact_local_hitbox_point: candidate.contact_local_hitbox_point,
            contact_world_hitbox_point: candidate.contact_world_hitbox_point,
            dodge_contact: candidate.dodge_contact,
        }
    }

    fn explicit_touch_event(
        &self,
        event: &TouchEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        allow_team_only_cache_attribution: bool,
    ) -> Option<TouchEvent> {
        if event.player.is_some() {
            Some(self.enrich_explicit_touch_event_from_current_frame(event, ball, players))
        } else if allow_team_only_cache_attribution {
            self.enrich_team_touch_event_from_recent_cache(event, ball, players)
        } else {
            None
        }
    }

    fn contested_touch_candidates(&self, primary: &TouchEvent) -> Vec<TouchEvent> {
        let primary_score = touch_candidate_score(
            primary.closest_approach_distance.unwrap_or(f32::INFINITY),
            primary.dodge_contact,
        );

        let mut opposing_candidates = self
            .recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 != primary.team_is_team_0)
            .filter(|candidate| {
                candidate.frame.abs_diff(primary.frame) <= CONTESTED_TOUCH_WINDOW_FRAMES
            })
            .filter(|candidate| {
                touch_candidate_score(
                    candidate.closest_approach_distance.unwrap_or(f32::INFINITY),
                    candidate.dodge_contact,
                ) <= primary_score + TOUCH_SCORING.contested_touch_score_margin
            })
            .cloned()
            .collect::<Vec<_>>();
        opposing_candidates.sort_by(touch_event_ordering);

        opposing_candidates
    }

    /// Credits cars that are physically on the ball but were never accepted as a
    /// deviation candidate, when an opposing-team touch was confirmed in the recent
    /// window. This recovers the losing side of a contested 50/50: the challenger
    /// reaches the ball a frame off the measurable trajectory deviation (which the
    /// winning car already produced), so it never caches and is otherwise dropped.
    fn geometric_contested_touches(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        confirmed_players: &HashSet<PlayerId>,
    ) -> Vec<TouchEvent> {
        if self.previous_ball_rigid_body.is_none() {
            return Vec::new();
        }
        let Some(ball) = ball.sample() else {
            return Vec::new();
        };

        players
            .players
            .iter()
            .filter(|player| !confirmed_players.contains(&player.player_id))
            .filter(|player| self.has_recent_opposing_team_touch(frame, player.is_team_0))
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let (closest_contact_gap, _current_contact_gap) =
                    touch_candidate_contact_gap_rank_with_hitbox(
                        &ball.rigid_body,
                        rigid_body,
                        player.hitbox,
                    )?;
                if closest_contact_gap > GEOMETRIC_CONTEST_MAX_GAP {
                    return None;
                }
                let contact_fields =
                    touch_event_contact_fields(ball.position(), rigid_body, player.hitbox);
                Some(TouchEvent {
                    touch_id: None,
                    time: frame.time,
                    frame: frame.frame_number,
                    team_is_team_0: player.is_team_0,
                    player: Some(player.player_id.clone()),
                    player_position: Some(rigid_body.location),
                    closest_approach_distance: Some(closest_contact_gap),
                    contact_local_ball_position: contact_fields.local_ball_position,
                    contact_local_hitbox_point: contact_fields.local_hitbox_point,
                    contact_world_hitbox_point: contact_fields.world_hitbox_point,
                    dodge_contact: player.dodge_active,
                })
            })
            .collect()
    }

    fn has_recent_opposing_team_touch(&self, frame: &FrameInfo, team_is_team_0: bool) -> bool {
        self.recent_team_touches.iter().any(|touch| {
            touch.team_is_team_0 != team_is_team_0
                && frame.frame_number.saturating_sub(touch.frame) <= GEOMETRIC_CONTEST_WINDOW_FRAMES
                && frame.time - touch.time <= GEOMETRIC_CONTEST_WINDOW_SECONDS
        })
    }

    fn confirmed_touch_events(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> Vec<TouchEvent> {
        let mut touch_events = Vec::new();
        let mut confirmed_players = HashSet::new();

        if let Some(ball_deviation) = self.ball_trajectory_deviation(frame, ball) {
            let candidate_events =
                self.candidate_touch_events(frame, ball, players, ball_deviation);
            if let Some(candidate) = candidate_events.first() {
                for contested_candidate in self.contested_touch_candidates(candidate) {
                    if let Some(player_id) = contested_candidate.player.as_ref() {
                        if confirmed_players.contains(player_id) {
                            continue;
                        }
                        confirmed_players.insert(player_id.clone());
                    }
                    touch_events.push(contested_candidate);
                }
                for candidate in candidate_events {
                    if let Some(player_id) = candidate.player.clone() {
                        if confirmed_players.contains(&player_id) {
                            continue;
                        }
                        confirmed_players.insert(player_id);
                    }
                    touch_events.push(candidate);
                }
            }
        }

        if touch_events.is_empty() {
            for event in &events.touch_events {
                let Some(event) = self.explicit_touch_event(event, ball, players, true) else {
                    continue;
                };
                if let Some(player_id) = event.player.clone() {
                    confirmed_players.insert(player_id);
                }
                touch_events.push(event);
            }
        } else {
            for event in &events.touch_events {
                if event.player.is_none() {
                    continue;
                }
                let Some(event) = self.explicit_touch_event(event, ball, players, false) else {
                    continue;
                };
                let Some(player_id) = event.player.clone() else {
                    continue;
                };
                if !confirmed_players.insert(player_id) {
                    continue;
                }
                touch_events.push(event);
            }
        }

        for dodge_refresh in &events.dodge_refreshed_events {
            if !confirmed_players.insert(dodge_refresh.player.clone()) {
                continue;
            }
            let Some(candidate) = self
                .candidate_for_player(&dodge_refresh.player)
                .or_else(|| {
                    self.current_frame_touch_candidate_for_dodge_refresh(
                        dodge_refresh,
                        ball,
                        players,
                    )
                })
            else {
                continue;
            };
            touch_events.push(Self::touch_event_from_dodge_refresh(
                dodge_refresh,
                candidate,
            ));
        }

        for contested in self.geometric_contested_touches(frame, ball, players, &confirmed_players)
        {
            if let Some(player_id) = contested.player.clone() {
                if !confirmed_players.insert(player_id) {
                    continue;
                }
            }
            touch_events.push(contested);
        }

        touch_events
    }

    fn record_recent_team_touches(&mut self, frame: usize, touch_events: &[TouchEvent]) {
        self.recent_team_touches
            .retain(|touch| frame.saturating_sub(touch.frame) <= GEOMETRIC_CONTEST_WINDOW_FRAMES);
        for event in touch_events {
            self.recent_team_touches.push(RecentTeamTouch {
                frame: event.frame,
                time: event.time,
                team_is_team_0: event.team_is_team_0,
            });
        }
    }

    fn touch_cooldown_key(event: &TouchEvent) -> TouchCooldownKey {
        event
            .player
            .clone()
            .map(TouchCooldownKey::Player)
            .unwrap_or(TouchCooldownKey::Team(event.team_is_team_0))
    }

    fn touch_cooldown_allows(&mut self, event: &TouchEvent) -> bool {
        const FLOAT_EPSILON: f32 = 0.0001;

        let key = Self::touch_cooldown_key(event);
        let allowed = self.last_touch_times.get(&key).is_none_or(|last| {
            // The rate limit dedupes repeat samples of one continuous contact
            // (a carry or dribble). A dodge-powered touch arriving on the heels
            // of a passive contact is a distinct mechanic (flip-reset
            // conversion, flick), not another sample of the same contact, so it
            // bypasses the limit. Repeated dodge touches still rate-limit
            // against each other.
            (event.dodge_contact && !last.dodge_contact)
                || event.time - last.time + FLOAT_EPSILON >= TOUCH_RATE_LIMIT_SECONDS
        });
        if allowed {
            self.last_touch_times.insert(
                key,
                LastCooldownTouch {
                    time: event.time,
                    dodge_contact: event.dodge_contact,
                },
            );
        }
        allowed
    }

    fn apply_touch_cooldown(&mut self, mut touch_events: Vec<TouchEvent>) -> Vec<TouchEvent> {
        touch_events.sort_by(touch_event_chronological_ordering);
        let mut accepted = touch_events
            .into_iter()
            .filter(|event| self.touch_cooldown_allows(event))
            .collect::<Vec<_>>();
        accepted.sort_by(touch_event_chronological_ordering);
        accepted
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
    ) -> TouchState {
        let touch_events = if live_play_state.counts_toward_player_motion() {
            self.prune_recent_touch_candidates(frame.frame_number);
            self.update_recent_touch_candidates(frame, ball, players);
            let touch_events = self.confirmed_touch_events(frame, ball, players, events);
            let mut touch_events = self.apply_touch_cooldown(touch_events);
            for event in &mut touch_events {
                event.touch_id = Some(self.next_touch_id);
                self.next_touch_id += 1;
            }
            self.record_recent_team_touches(frame.frame_number, &touch_events);
            touch_events
        } else {
            self.current_last_touch = None;
            self.recent_touch_candidates.clear();
            self.last_touch_times.clear();
            self.recent_team_touches.clear();
            Vec::new()
        };

        if let Some(last_touch) = primary_touch_event(&touch_events) {
            self.current_last_touch = Some(last_touch.clone());
        }
        self.previous_ball_rigid_body = ball.sample().map(|sample| (sample.rigid_body, frame.time));

        TouchState {
            touch_events,
            last_touch: self.current_last_touch.clone(),
            last_touch_player: self
                .current_last_touch
                .as_ref()
                .and_then(|touch| touch.player.clone()),
            last_touch_team_is_team_0: self
                .current_last_touch
                .as_ref()
                .map(|touch| touch.team_is_team_0),
        }
    }
}

#[cfg(test)]
#[path = "touch_state_tests.rs"]
mod tests;
