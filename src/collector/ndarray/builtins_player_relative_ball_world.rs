use super::*;

build_player_feature_adder!(
    PlayerRelativeBallPosition,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        let relative_position = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                vec_to_glam(&ball_rigid_body.location) - vec_to_glam(&player_rigid_body.location)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            relative_position.x,
            relative_position.y,
            relative_position.z
        )
    },
    "relative ball position x",
    "relative ball position y",
    "relative ball position z"
);

build_player_feature_adder!(
    PlayerRelativeBallVelocity,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        let relative_velocity = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                vec_to_glam(
                    &ball_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                ) - vec_to_glam(
                    &player_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                )
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            relative_velocity.x,
            relative_velocity.y,
            relative_velocity.z
        )
    },
    "relative ball velocity x",
    "relative ball velocity y",
    "relative ball velocity z"
);
