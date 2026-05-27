use super::*;

const BACKBOARD_MIN_BALL_Z: f32 = 500.0;
const BACKBOARD_MIN_NORMALIZED_Y: f32 = 4700.0;
const BACKBOARD_MAX_ABS_X: f32 = 1600.0;
const BACKBOARD_MIN_APPROACH_SPEED_Y: f32 = 350.0;
const BACKBOARD_MIN_REBOUND_SPEED_Y: f32 = 250.0;
const BACKBOARD_TOUCH_ATTRIBUTION_MAX_SECONDS: f32 = 2.5;

impl BackboardBounceCalculator {
    pub(super) fn detect_bounce(
        &self,
        frame: &FrameInfo,
        ball: Option<&BallSample>,
        touch_events: &[TouchEvent],
    ) -> Option<BackboardBounceEvent> {
        if !touch_events.is_empty() {
            return None;
        }

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
        if current_normalized_velocity_y > -BACKBOARD_MIN_REBOUND_SPEED_Y {
            return None;
        }

        Some(BackboardBounceEvent {
            time: frame.time,
            frame: frame.frame_number,
            player,
            is_team_0: last_touch.team_is_team_0,
        })
    }
}
