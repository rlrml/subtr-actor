use super::*;

build_player_feature_adder!(
    PlayerBallDistance,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        let distance = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                (vec_to_glam(&player_rigid_body.location) - vec_to_glam(&ball_rigid_body.location))
                    .length()
            })
            .unwrap_or(0.0);
        convert_all_floats!(distance)
    },
    "distance to ball"
);
