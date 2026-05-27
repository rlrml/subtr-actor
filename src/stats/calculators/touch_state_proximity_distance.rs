use super::*;

const OCTANE_HITBOX_LENGTH: f32 = 118.01;
const OCTANE_HITBOX_WIDTH: f32 = 84.2;
const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
const OCTANE_HITBOX_OFFSET: f32 = 13.88;
const OCTANE_HITBOX_ELEVATION: f32 = 17.05;

pub(crate) fn collision_distance(player: &PlayerSample, ball_position: glam::Vec3) -> Option<f32> {
    let rigid_body = player.rigid_body.as_ref()?;
    let player_position = vec_to_glam(&rigid_body.location);
    let local_ball_position =
        quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position);

    Some(
        glam::Vec3::new(
            axis_distance(
                local_ball_position.x,
                -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET,
                OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET,
            ),
            axis_distance(
                local_ball_position.y,
                -OCTANE_HITBOX_WIDTH / 2.0,
                OCTANE_HITBOX_WIDTH / 2.0,
            ),
            axis_distance(
                local_ball_position.z,
                -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION,
                OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION,
            ),
        )
        .length(),
    )
}

fn axis_distance(value: f32, min_value: f32, max_value: f32) -> f32 {
    if value < min_value {
        min_value - value
    } else if value > max_value {
        value - max_value
    } else {
        0.0
    }
}
