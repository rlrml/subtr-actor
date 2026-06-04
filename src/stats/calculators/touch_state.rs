use super::*;

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

const TOUCH_SCORING: TouchCandidateScoring = TouchCandidateScoring::DEFAULT;
const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;
const CONTESTED_TOUCH_WINDOW_FRAMES: usize = 1;
const BALL_GRAVITY_Z: f32 = -650.0;

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

#[derive(Debug, Clone, Default)]
pub struct TouchStateCalculator {
    previous_ball_rigid_body: Option<(boxcars::RigidBody, f32)>,
    current_last_touch: Option<TouchEvent>,
    recent_touch_candidates: HashMap<PlayerId, TouchEvent>,
    last_touch_times: HashMap<TouchCooldownKey, f32>,
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

                Some(TouchEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    team_is_team_0: player.is_team_0,
                    player: Some(player.player_id.clone()),
                    player_position: Some(rigid_body.location),
                    closest_approach_distance: Some(closest_contact_gap),
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

    fn enrich_team_touch_event_from_recent_cache(&self, event: &TouchEvent) -> Option<TouchEvent> {
        let candidate = self.best_candidate_for_team(event.team_is_team_0)?;

        Some(TouchEvent {
            time: event.time,
            frame: event.frame,
            team_is_team_0: event.team_is_team_0,
            player: event.player.clone().or(candidate.player),
            player_position: event.player_position.or(candidate.player_position),
            closest_approach_distance: event
                .closest_approach_distance
                .or(candidate.closest_approach_distance),
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

        TouchEvent {
            team_is_team_0: player.is_team_0,
            player_position: event
                .player_position
                .or_else(|| rigid_body.map(|rigid_body| rigid_body.location)),
            closest_approach_distance: event.closest_approach_distance.or(closest_contact_gap),
            dodge_contact: event.dodge_contact || player.dodge_active,
            ..event.clone()
        }
    }

    fn touch_event_from_dodge_refresh(
        dodge_refresh: &DodgeRefreshedEvent,
        candidate: TouchEvent,
    ) -> TouchEvent {
        TouchEvent {
            time: dodge_refresh.time,
            frame: dodge_refresh.frame,
            team_is_team_0: dodge_refresh.is_team_0,
            player: Some(dodge_refresh.player.clone()),
            player_position: dodge_refresh
                .player_position
                .map(|position| glam_to_vec(&glam::Vec3::from_array(position)))
                .or(candidate.player_position),
            closest_approach_distance: candidate.closest_approach_distance,
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
            self.enrich_team_touch_event_from_recent_cache(event)
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
            let Some(candidate) = self.candidate_for_player(&dodge_refresh.player) else {
                continue;
            };
            touch_events.push(Self::touch_event_from_dodge_refresh(
                dodge_refresh,
                candidate,
            ));
        }

        touch_events
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
        let allowed = self.last_touch_times.get(&key).is_none_or(|last_time| {
            event.time - last_time + FLOAT_EPSILON >= TOUCH_RATE_LIMIT_SECONDS
        });
        if allowed {
            self.last_touch_times.insert(key, event.time);
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
        let touch_events = if live_play_state.is_live_play {
            self.prune_recent_touch_candidates(frame.frame_number);
            self.update_recent_touch_candidates(frame, ball, players);
            let touch_events = self.confirmed_touch_events(frame, ball, players, events);
            self.apply_touch_cooldown(touch_events)
        } else {
            self.current_last_touch = None;
            self.recent_touch_candidates.clear();
            self.last_touch_times.clear();
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
