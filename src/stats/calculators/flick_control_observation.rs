use super::*;

impl FlickCalculator {
    pub(super) fn ball_impulse(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> glam::Vec3 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return glam::Vec3::ZERO;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return glam::Vec3::ZERO;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        ball.velocity() - previous_ball_velocity - expected_linear_delta
    }

    pub(super) fn control_observation(
        ball: &BallSample,
        player: &PlayerSample,
        controlling_player: Option<&PlayerId>,
    ) -> Option<FlickControlObservation> {
        if controlling_player != Some(&player.player_id) {
            return None;
        }

        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let ball_position = ball.position();
        if !(BALL_CARRY_MIN_BALL_Z..=FLICK_MAX_CONTROL_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        if horizontal_gap > FLICK_MAX_CONTROL_HORIZONTAL_GAP {
            return None;
        }

        let vertical_gap = ball_position.z - player_position.z;
        if !(FLICK_MIN_CONTROL_VERTICAL_GAP..=FLICK_MAX_CONTROL_VERTICAL_GAP)
            .contains(&vertical_gap)
        {
            return None;
        }

        let local_ball_position =
            quat_to_glam(&player_rigid_body.rotation).inverse() * (ball_position - player_position);
        if local_ball_position.x < -FLICK_MAX_LOCAL_X_BEHIND
            || local_ball_position.x > FLICK_MAX_LOCAL_X_FRONT
            || local_ball_position.y.abs() > FLICK_MAX_LOCAL_Y
            || local_ball_position.z < FLICK_MIN_LOCAL_Z
        {
            return None;
        }

        Some(FlickControlObservation {
            horizontal_gap,
            vertical_gap,
        })
    }
}
