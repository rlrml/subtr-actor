use super::*;

impl DodgeResetCalculator {
    pub(super) fn player_is_grounded(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id)
            .and_then(PlayerSample::position)
            .is_some_and(|position| position.z <= FLIP_RESET_GROUNDED_Z)
    }

    pub(super) fn player_dodge_active(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id).is_some_and(|player| player.dodge_active)
    }

    pub(super) fn on_ball_dodge_reset(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> bool {
        const MIN_PLAYER_HEIGHT: f32 = 95.0;
        const MIN_BALL_HEIGHT: f32 = 80.0;
        const MAX_CENTER_DISTANCE: f32 = 180.0;
        const MAX_LOCAL_VERTICAL_OFFSET: f32 = 140.0;

        let Some(ball) = ball.sample() else {
            return false;
        };
        let Some(player) = Self::player(players, player_id) else {
            return false;
        };
        let Some(player_rigid_body) = &player.rigid_body else {
            return false;
        };

        let ball_position = vec_to_glam(&ball.rigid_body.location);
        let player_position = vec_to_glam(&player_rigid_body.location);
        if player_position.z < MIN_PLAYER_HEIGHT || ball_position.z < MIN_BALL_HEIGHT {
            return false;
        }

        let relative_ball_position = ball_position - player_position;
        let center_distance = relative_ball_position.length();
        if !center_distance.is_finite() || center_distance > MAX_CENTER_DISTANCE {
            return false;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        local_ball_position.z <= MAX_LOCAL_VERTICAL_OFFSET
    }
}
