use super::*;

impl BallCarryCalculator {
    pub(crate) fn carry_frame_sample(
        player: &PlayerSample,
        ball: &BallSample,
    ) -> Option<ContinuousBallControlSample<BallCarryKind>> {
        let player_position = player.position()?;
        let ball_position = ball.position();
        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        let vertical_gap = ball_position.z - player_position.z;

        if AirDribblePolicy::is_sample(player_position, ball_position, horizontal_gap, vertical_gap)
        {
            return Some(control_sample(
                player,
                player_position,
                BallCarryKind::AirDribble,
                horizontal_gap,
                vertical_gap,
            ));
        }

        if player_is_on_wall(player_position)
            || !(BALL_CARRY_MIN_BALL_Z..=BALL_CARRY_MAX_BALL_Z).contains(&ball_position.z)
            || horizontal_gap > BALL_CARRY_MAX_HORIZONTAL_GAP
            || !(0.0..=BALL_CARRY_MAX_VERTICAL_GAP).contains(&vertical_gap)
        {
            return None;
        }

        Some(control_sample(
            player,
            player_position,
            BallCarryKind::Carry,
            horizontal_gap,
            vertical_gap,
        ))
    }
}

fn control_sample(
    player: &PlayerSample,
    player_position: glam::Vec3,
    kind: BallCarryKind,
    horizontal_gap: f32,
    vertical_gap: f32,
) -> ContinuousBallControlSample<BallCarryKind> {
    ContinuousBallControlSample {
        player_position,
        kind,
        horizontal_gap,
        vertical_gap,
        speed: player.speed().unwrap_or(0.0),
    }
}
