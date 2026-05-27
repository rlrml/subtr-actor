use super::*;

build_player_feature_adder!(
    PlayerLocalRelativeBallPosition,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        let local_relative_position = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                let player_rotation = player_rigid_body.rotation;
                let player_quat = glam::quat(
                    player_rotation.x,
                    player_rotation.y,
                    player_rotation.z,
                    player_rotation.w,
                );
                let world_relative_position = vec_to_glam(&ball_rigid_body.location)
                    - vec_to_glam(&player_rigid_body.location);
                player_quat.inverse().mul_vec3(world_relative_position)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            local_relative_position.x,
            local_relative_position.y,
            local_relative_position.z
        )
    },
    "local relative ball position x",
    "local relative ball position y",
    "local relative ball position z"
);

build_player_feature_adder!(
    PlayerLocalRelativeBallVelocity,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        let local_relative_velocity = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                let player_rotation = player_rigid_body.rotation;
                let player_quat = glam::quat(
                    player_rotation.x,
                    player_rotation.y,
                    player_rotation.z,
                    player_rotation.w,
                );
                let world_relative_velocity = vec_to_glam(
                    &ball_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                ) - vec_to_glam(
                    &player_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                );
                player_quat.inverse().mul_vec3(world_relative_velocity)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            local_relative_velocity.x,
            local_relative_velocity.y,
            local_relative_velocity.z
        )
    },
    "local relative ball velocity x",
    "local relative ball velocity y",
    "local relative ball velocity z"
);
