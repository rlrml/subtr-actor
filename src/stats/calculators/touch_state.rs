use super::*;

#[derive(Debug, Clone, Default)]
pub struct TouchState {
    pub touch_events: Vec<TouchEvent>,
    pub last_touch: Option<TouchEvent>,
    pub last_touch_player: Option<PlayerId>,
    pub last_touch_team_is_team_0: Option<bool>,
}

#[derive(Default)]
pub struct TouchStateCalculator {
    previous_ball_linear_velocity: Option<glam::Vec3>,
    previous_ball_angular_velocity: Option<glam::Vec3>,
    current_last_touch: Option<TouchEvent>,
    recent_touch_candidates: HashMap<PlayerId, TouchEvent>,
}

impl TouchStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    fn should_emit_candidate(&self, candidate: &TouchEvent) -> bool {
        const SAME_PLAYER_TOUCH_COOLDOWN_FRAMES: usize = 7;

        let Some(previous_touch) = self.current_last_touch.as_ref() else {
            return true;
        };

        let same_player =
            previous_touch.player.is_some() && previous_touch.player == candidate.player;
        if !same_player {
            return true;
        }

        candidate.frame.saturating_sub(previous_touch.frame) >= SAME_PLAYER_TOUCH_COOLDOWN_FRAMES
    }

    fn prune_recent_touch_candidates(&mut self, current_frame: usize) {
        const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;

        self.recent_touch_candidates.retain(|_, candidate| {
            current_frame.saturating_sub(candidate.frame) <= TOUCH_CANDIDATE_WINDOW_FRAMES
        });
    }

    fn current_ball_angular_velocity(ball: &BallFrameState) -> Option<glam::Vec3> {
        ball.sample()
            .map(|ball| {
                ball.rigid_body
                    .angular_velocity
                    .unwrap_or(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    })
            })
            .map(|velocity| vec_to_glam(&velocity))
    }

    fn current_ball_linear_velocity(ball: &BallFrameState) -> Option<glam::Vec3> {
        ball.velocity()
    }

    fn is_touch_candidate(&self, frame: &FrameInfo, ball: &BallFrameState) -> bool {
        const BALL_GRAVITY_Z: f32 = -650.0;
        const TOUCH_LINEAR_IMPULSE_THRESHOLD: f32 = 120.0;
        const TOUCH_ANGULAR_VELOCITY_DELTA_THRESHOLD: f32 = 0.5;

        let Some(current_linear_velocity) = Self::current_ball_linear_velocity(ball) else {
            return false;
        };
        let Some(previous_linear_velocity) = self.previous_ball_linear_velocity else {
            return false;
        };
        let Some(current_angular_velocity) = Self::current_ball_angular_velocity(ball) else {
            return false;
        };
        let Some(previous_angular_velocity) = self.previous_ball_angular_velocity else {
            return false;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        let residual_linear_impulse =
            current_linear_velocity - previous_linear_velocity - expected_linear_delta;
        let angular_velocity_delta = current_angular_velocity - previous_angular_velocity;

        residual_linear_impulse.length() > TOUCH_LINEAR_IMPULSE_THRESHOLD
            || angular_velocity_delta.length() > TOUCH_ANGULAR_VELOCITY_DELTA_THRESHOLD
    }

    fn proximity_touch_candidates(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        max_collision_distance: f32,
    ) -> Vec<TouchEvent> {
        const OCTANE_HITBOX_LENGTH: f32 = 118.01;
        const OCTANE_HITBOX_WIDTH: f32 = 84.2;
        const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
        const OCTANE_HITBOX_OFFSET: f32 = 13.88;
        const OCTANE_HITBOX_ELEVATION: f32 = 17.05;

        let Some(ball) = ball.sample() else {
            return Vec::new();
        };
        let ball_position = vec_to_glam(&ball.rigid_body.location);

        let mut candidates = players
            .players
            .iter()
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let player_position = vec_to_glam(&rigid_body.location);
                let local_ball_position = quat_to_glam(&rigid_body.rotation).inverse()
                    * (ball_position - player_position);

                let x_distance = if local_ball_position.x
                    < -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET
                {
                    (-OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET) - local_ball_position.x
                } else if local_ball_position.x > OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET
                {
                    local_ball_position.x - (OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET)
                } else {
                    0.0
                };
                let y_distance = if local_ball_position.y < -OCTANE_HITBOX_WIDTH / 2.0 {
                    (-OCTANE_HITBOX_WIDTH / 2.0) - local_ball_position.y
                } else if local_ball_position.y > OCTANE_HITBOX_WIDTH / 2.0 {
                    local_ball_position.y - OCTANE_HITBOX_WIDTH / 2.0
                } else {
                    0.0
                };
                let z_distance = if local_ball_position.z
                    < -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION
                {
                    (-OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION) - local_ball_position.z
                } else if local_ball_position.z
                    > OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION
                {
                    local_ball_position.z - (OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION)
                } else {
                    0.0
                };

                let collision_distance =
                    glam::Vec3::new(x_distance, y_distance, z_distance).length();
                if collision_distance > max_collision_distance {
                    return None;
                }

                Some(TouchEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    team_is_team_0: player.is_team_0,
                    player: Some(player.player_id.clone()),
                    closest_approach_distance: Some(collision_distance),
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            let left_distance = left.closest_approach_distance.unwrap_or(f32::INFINITY);
            let right_distance = right.closest_approach_distance.unwrap_or(f32::INFINITY);
            left_distance.total_cmp(&right_distance)
        });
        candidates
    }

    fn candidate_touch_event(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<TouchEvent> {
        const TOUCH_COLLISION_DISTANCE_THRESHOLD: f32 = 300.0;

        self.proximity_touch_candidates(frame, ball, players, TOUCH_COLLISION_DISTANCE_THRESHOLD)
            .into_iter()
            .next()
    }

    fn update_recent_touch_candidates(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        const PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD: f32 = 220.0;

        for candidate in self.proximity_touch_candidates(
            frame,
            ball,
            players,
            PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD,
        ) {
            let Some(player_id) = candidate.player.clone() else {
                continue;
            };

            self.recent_touch_candidates.insert(player_id, candidate);
        }
    }

    fn candidate_for_player(&self, player_id: &PlayerId) -> Option<TouchEvent> {
        self.recent_touch_candidates.get(player_id).cloned()
    }

    fn contested_touch_candidates(&self, primary: &TouchEvent) -> Vec<TouchEvent> {
        const CONTESTED_TOUCH_DISTANCE_MARGIN: f32 = 80.0;

        let primary_distance = primary.closest_approach_distance.unwrap_or(f32::INFINITY);

        let best_opposing_candidate = self
            .recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 != primary.team_is_team_0)
            .filter(|candidate| {
                candidate.closest_approach_distance.unwrap_or(f32::INFINITY)
                    <= primary_distance + CONTESTED_TOUCH_DISTANCE_MARGIN
            })
            .min_by(|left, right| {
                let left_distance = left.closest_approach_distance.unwrap_or(f32::INFINITY);
                let right_distance = right.closest_approach_distance.unwrap_or(f32::INFINITY);
                left_distance.total_cmp(&right_distance)
            })
            .cloned();

        best_opposing_candidate.into_iter().collect()
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

        if self.is_touch_candidate(frame, ball) {
            if let Some(candidate) = self.candidate_touch_event(frame, ball, players) {
                for contested_candidate in self.contested_touch_candidates(&candidate) {
                    if let Some(player_id) = contested_candidate.player.clone() {
                        confirmed_players.insert(player_id);
                    }
                    touch_events.push(contested_candidate);
                }
                if let Some(player_id) = candidate.player.clone() {
                    confirmed_players.insert(player_id);
                }
                touch_events.push(candidate);
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
            self.confirmed_touch_events(frame, ball, players, events)
                .into_iter()
                .filter(|candidate| self.should_emit_candidate(candidate))
                .collect()
        } else {
            self.current_last_touch = None;
            self.recent_touch_candidates.clear();
            Vec::new()
        };

        if let Some(last_touch) = touch_events.last() {
            self.current_last_touch = Some(last_touch.clone());
        }
        self.previous_ball_linear_velocity = Self::current_ball_linear_velocity(ball);
        self.previous_ball_angular_velocity = Self::current_ball_angular_velocity(ball);

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
