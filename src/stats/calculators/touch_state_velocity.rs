use super::*;

impl TouchStateCalculator {
    pub(super) fn current_ball_angular_velocity(ball: &BallFrameState) -> Option<glam::Vec3> {
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

    pub(super) fn current_ball_linear_velocity(ball: &BallFrameState) -> Option<glam::Vec3> {
        ball.velocity()
    }

    pub(super) fn is_touch_candidate(&self, frame: &FrameInfo, ball: &BallFrameState) -> bool {
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
}
