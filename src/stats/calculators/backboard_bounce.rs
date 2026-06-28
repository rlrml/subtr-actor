use super::*;

/// A ball rebound off the opponent backboard attributed to the player who sent it there.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardBounceEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
}

#[cfg(test)]
#[path = "backboard_bounce_tests.rs"]
mod tests;

/// Per-frame tracking state for backboard bounces.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackboardBounceState {
    pub bounce_events: Vec<BackboardBounceEvent>,
    pub last_bounce_event: Option<BackboardBounceEvent>,
}

/// Detects backboard bounces and attributes them to the player who sent the ball.
#[derive(Default)]
pub struct BackboardBounceCalculator {
    previous_ball_velocity: Option<glam::Vec3>,
    last_touch: Option<TouchEvent>,
    last_bounce_event: Option<BackboardBounceEvent>,
}

impl BackboardBounceCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    fn detect_bounce(
        &self,
        frame: &FrameInfo,
        ball: Option<&BallSample>,
        touch_state: &TouchState,
    ) -> Option<BackboardBounceEvent> {
        const BACKBOARD_MIN_BALL_Z: f32 = 500.0;
        const BACKBOARD_MIN_NORMALIZED_Y: f32 = 4700.0;
        const BACKBOARD_SIMULTANEOUS_TOUCH_MIN_NORMALIZED_Y: f32 = 5000.0;
        const BACKBOARD_MAX_ABS_X: f32 = 1600.0;
        const BACKBOARD_MIN_APPROACH_SPEED_Y: f32 = 350.0;
        const BACKBOARD_MIN_REBOUND_SPEED_Y: f32 = 250.0;
        const BACKBOARD_TOUCH_ATTRIBUTION_MAX_SECONDS: f32 = 2.5;

        let last_touch = self.last_touch.as_ref()?;
        let player = last_touch.player.clone()?;
        let current_ball = ball?;
        let previous_ball_velocity = self.previous_ball_velocity?;

        if (frame.time - last_touch.time).max(0.0) > BACKBOARD_TOUCH_ATTRIBUTION_MAX_SECONDS {
            return None;
        }

        let ball_position = current_ball.position();
        if ball_position.x.abs() > BACKBOARD_MAX_ABS_X || ball_position.z < BACKBOARD_MIN_BALL_Z {
            return None;
        }

        let normalized_position_y = normalized_y(last_touch.team_is_team_0, ball_position);
        if normalized_position_y < BACKBOARD_MIN_NORMALIZED_Y {
            return None;
        }

        let previous_normalized_velocity_y = if last_touch.team_is_team_0 {
            previous_ball_velocity.y
        } else {
            -previous_ball_velocity.y
        };
        let current_normalized_velocity_y = if last_touch.team_is_team_0 {
            current_ball.velocity().y
        } else {
            -current_ball.velocity().y
        };

        if previous_normalized_velocity_y < BACKBOARD_MIN_APPROACH_SPEED_Y {
            return None;
        }

        let has_rebound_velocity = current_normalized_velocity_y <= -BACKBOARD_MIN_REBOUND_SPEED_Y;
        let has_simultaneous_same_player_touch =
            touch_state.primary_touch_event().is_some_and(|touch| {
                touch.team_is_team_0 == last_touch.team_is_team_0
                    && touch.player.as_ref() == Some(&player)
                    && normalized_position_y >= BACKBOARD_SIMULTANEOUS_TOUCH_MIN_NORMALIZED_Y
                    && (touch.frame > last_touch.frame
                        || (touch.frame == last_touch.frame && touch.time > last_touch.time))
            });
        if !has_rebound_velocity && !has_simultaneous_same_player_touch {
            return None;
        }

        Some(BackboardBounceEvent {
            time: frame.time,
            frame: frame.frame_number,
            player,
            player_position: last_touch
                .player_position
                .map(|position| vec_to_glam(&position).to_array()),
            is_team_0: last_touch.team_is_team_0,
        })
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> BackboardBounceState {
        if !live_play_state.is_live_play {
            self.previous_ball_velocity = ball.velocity();
            self.last_touch = None;
            self.last_bounce_event = None;
            return BackboardBounceState::default();
        }

        if self
            .last_touch
            .as_ref()
            .and_then(|touch| touch.player.as_ref())
            .and_then(|player_id| players.player(player_id))
            .is_some_and(player_sample_is_touching_surface)
        {
            self.last_touch = None;
        }

        let bounce_events: Vec<_> = self
            .detect_bounce(frame, ball.sample(), touch_state)
            .into_iter()
            .collect();
        if let Some(last_bounce_event) = bounce_events.last() {
            self.last_bounce_event = Some(last_bounce_event.clone());
        }

        if !touch_state.touch_events.is_empty() {
            self.last_touch = touch_state.primary_touch_event().cloned();
        }
        self.previous_ball_velocity = ball.velocity();

        BackboardBounceState {
            bounce_events,
            last_bounce_event: self.last_bounce_event.clone(),
        }
    }
}
