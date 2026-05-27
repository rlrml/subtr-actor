use super::*;

impl WhiffCalculator {
    pub(super) fn hitbox_distance(ball_position: glam::Vec3, player: &PlayerSample) -> Option<f32> {
        const OCTANE_HITBOX_LENGTH: f32 = 118.01;
        const OCTANE_HITBOX_WIDTH: f32 = 84.2;
        const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
        const OCTANE_HITBOX_OFFSET: f32 = 13.88;
        const OCTANE_HITBOX_ELEVATION: f32 = 17.05;

        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let local_ball_position =
            quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position);

        let x_min = -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET;
        let x_max = OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET;
        let y_min = -OCTANE_HITBOX_WIDTH / 2.0;
        let y_max = OCTANE_HITBOX_WIDTH / 2.0;
        let z_min = -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION;
        let z_max = OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION;

        let x_distance = if local_ball_position.x < x_min {
            x_min - local_ball_position.x
        } else if local_ball_position.x > x_max {
            local_ball_position.x - x_max
        } else {
            0.0
        };
        let y_distance = if local_ball_position.y < y_min {
            y_min - local_ball_position.y
        } else if local_ball_position.y > y_max {
            local_ball_position.y - y_max
        } else {
            0.0
        };
        let z_distance = if local_ball_position.z < z_min {
            z_min - local_ball_position.z
        } else if local_ball_position.z > z_max {
            local_ball_position.z - z_max
        } else {
            0.0
        };

        Some(glam::Vec3::new(x_distance, y_distance, z_distance).length())
    }

    pub(super) fn local_ball_position(
        ball_position: glam::Vec3,
        player: &PlayerSample,
    ) -> Option<glam::Vec3> {
        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        Some(quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position))
    }
}
