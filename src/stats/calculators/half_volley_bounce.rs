use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FloorBounce {
    pub(super) time: f32,
    pub(super) frame: usize,
}

impl HalfVolleyCalculator {
    pub(super) fn detect_floor_bounce(
        frame: &FrameInfo,
        ball: Option<&BallSample>,
        previous_ball_velocity: Option<glam::Vec3>,
        touch_events: &[TouchEvent],
    ) -> Option<FloorBounce> {
        if !touch_events.is_empty() {
            return None;
        }
        let ball = ball?;
        let previous_ball_velocity = previous_ball_velocity?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        if ball_position.z > HALF_VOLLEY_FLOOR_BOUNCE_MAX_BALL_Z
            || previous_ball_velocity.z > -HALF_VOLLEY_FLOOR_BOUNCE_MIN_APPROACH_SPEED_Z
            || ball_velocity.z < HALF_VOLLEY_FLOOR_BOUNCE_MIN_REBOUND_SPEED_Z
        {
            return None;
        }

        Some(FloorBounce {
            time: frame.time,
            frame: frame.frame_number,
        })
    }
}
