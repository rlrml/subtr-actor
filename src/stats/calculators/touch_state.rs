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
        self.last_touch
            .as_ref()
            .filter(|primary| self.touch_events.iter().any(|event| event == *primary))
            .or_else(|| self.touch_events.last())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TouchCooldownKey {
    Player(PlayerId),
    Team(bool),
}

const TOUCH_SCORING: TouchCandidateScoring = TouchCandidateScoring::DEFAULT;
const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;
const BALL_GRAVITY_Z: f32 = -650.0;

#[derive(Clone, Default)]
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

    fn accepted_contact_gap(closest_contact_gap: f32, velocity_deviation: f32) -> bool {
        TOUCH_SCORING.accepts_contact_gap(closest_contact_gap, velocity_deviation)
    }

    fn touch_candidate_score(closest_contact_gap: f32, dodge_contact: bool) -> f32 {
        TOUCH_SCORING.score_contact_gap(closest_contact_gap, dodge_contact)
    }

    fn touch_event_score(event: &TouchEvent) -> f32 {
        Self::touch_candidate_score(
            event.closest_approach_distance.unwrap_or(f32::INFINITY),
            event.dodge_contact,
        )
    }

    fn primary_touch_event(touch_events: &[TouchEvent]) -> Option<TouchEvent> {
        touch_events
            .iter()
            .min_by(|left, right| {
                Self::touch_event_score(left).total_cmp(&Self::touch_event_score(right))
            })
            .cloned()
    }

    fn prune_recent_touch_candidates(&mut self, current_frame: usize) {
        self.recent_touch_candidates.retain(|_, candidate| {
            current_frame.saturating_sub(candidate.frame) <= TOUCH_CANDIDATE_WINDOW_FRAMES
        });
    }

    fn ball_velocity_deviation(&self, frame: &FrameInfo, ball: &BallFrameState) -> Option<f32> {
        let current_ball = ball.sample()?;
        let (previous_ball, previous_time) = &self.previous_ball_rigid_body?;
        ball_trajectory_deviation_with_gravity(
            previous_ball,
            *previous_time,
            &current_ball.rigid_body,
            frame.time,
            BALL_GRAVITY_Z,
        )
        .map(|deviation| deviation.velocity_deviation)
    }

    fn proximity_touch_candidates(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        velocity_deviation: f32,
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
                if !Self::accepted_contact_gap(closest_contact_gap, velocity_deviation) {
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

        candidates.sort_by(|left, right| {
            let left_score = Self::touch_candidate_score(
                left.closest_approach_distance.unwrap_or(f32::INFINITY),
                left.dodge_contact,
            );
            let right_score = Self::touch_candidate_score(
                right.closest_approach_distance.unwrap_or(f32::INFINITY),
                right.dodge_contact,
            );
            left_score.total_cmp(&right_score)
        });
        candidates
    }

    fn candidate_touch_events(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        velocity_deviation: f32,
    ) -> Vec<TouchEvent> {
        let candidates = self.proximity_touch_candidates(frame, ball, players, velocity_deviation);
        let Some(primary) = candidates.first() else {
            return Vec::new();
        };
        let primary_score = Self::touch_candidate_score(
            primary.closest_approach_distance.unwrap_or(f32::INFINITY),
            primary.dodge_contact,
        );
        candidates
            .into_iter()
            .filter(|candidate| {
                let score = Self::touch_candidate_score(
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
        let Some(velocity_deviation) = self.ball_velocity_deviation(frame, ball) else {
            return;
        };

        for candidate in self.proximity_touch_candidates(frame, ball, players, velocity_deviation) {
            let Some(player_id) = candidate.player.clone() else {
                continue;
            };

            if self
                .recent_touch_candidates
                .get(&player_id)
                .is_none_or(|previous| {
                    Self::touch_event_score(&candidate) < Self::touch_event_score(previous)
                })
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
            .min_by(|left, right| {
                let left_score = Self::touch_candidate_score(
                    left.closest_approach_distance.unwrap_or(f32::INFINITY),
                    left.dodge_contact,
                );
                let right_score = Self::touch_candidate_score(
                    right.closest_approach_distance.unwrap_or(f32::INFINITY),
                    right.dodge_contact,
                );
                left_score.total_cmp(&right_score)
            })
            .cloned()
    }

    fn enrich_explicit_touch_event(&self, event: &TouchEvent) -> Option<TouchEvent> {
        let candidate = if let Some(player_id) = event.player.as_ref() {
            self.candidate_for_player(player_id)
        } else {
            self.best_candidate_for_team(event.team_is_team_0)
        };
        let candidate = candidate?;

        Some(TouchEvent {
            player: event.player.clone().or(candidate.player),
            player_position: event.player_position.or(candidate.player_position),
            closest_approach_distance: event
                .closest_approach_distance
                .or(candidate.closest_approach_distance),
            dodge_contact: event.dodge_contact || candidate.dodge_contact,
            ..candidate
        })
    }

    fn contested_touch_candidates(&self, primary: &TouchEvent) -> Vec<TouchEvent> {
        let primary_score = Self::touch_candidate_score(
            primary.closest_approach_distance.unwrap_or(f32::INFINITY),
            primary.dodge_contact,
        );

        let mut opposing_candidates = self
            .recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 != primary.team_is_team_0)
            .filter(|candidate| {
                Self::touch_candidate_score(
                    candidate.closest_approach_distance.unwrap_or(f32::INFINITY),
                    candidate.dodge_contact,
                ) <= primary_score + TOUCH_SCORING.contested_touch_score_margin
            })
            .cloned()
            .collect::<Vec<_>>();
        opposing_candidates.sort_by(|left, right| {
            let left_score = Self::touch_candidate_score(
                left.closest_approach_distance.unwrap_or(f32::INFINITY),
                left.dodge_contact,
            );
            let right_score = Self::touch_candidate_score(
                right.closest_approach_distance.unwrap_or(f32::INFINITY),
                right.dodge_contact,
            );
            left_score.total_cmp(&right_score)
        });

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

        if let Some(velocity_deviation) = self.ball_velocity_deviation(frame, ball) {
            let candidate_events =
                self.candidate_touch_events(frame, ball, players, velocity_deviation);
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
                let event = if let Some(event) = self.enrich_explicit_touch_event(event) {
                    event
                } else if event.player.is_some() {
                    event.clone()
                } else {
                    continue;
                };
                if let Some(player_id) = event.player.clone() {
                    confirmed_players.insert(player_id);
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
            touch_events.push(candidate);
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

    fn apply_touch_cooldown(&mut self, touch_events: Vec<TouchEvent>) -> Vec<TouchEvent> {
        touch_events
            .into_iter()
            .filter(|event| self.touch_cooldown_allows(event))
            .collect()
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

        if let Some(last_touch) = Self::primary_touch_event(&touch_events) {
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
