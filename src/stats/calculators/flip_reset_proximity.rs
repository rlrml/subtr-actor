use super::*;

pub(crate) fn flip_reset_proximity_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
) -> Option<FlipResetHeuristic> {
    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);
    let center_distance = (raw_ball_position - raw_player_position).length() * scale_factor;
    let features = build_touch_features(ball_body, player_body, center_distance / scale_factor)?;
    if features.player_position.z < 95.0 || features.ball_position.z < 80.0 {
        return None;
    }
    if features.center_distance > 110.0
        || features.underside_alignment < 0.52
        || features.local_ball_position.x.abs() > 260.0
        || features.local_ball_position.y.abs() > 260.0
        || features.local_ball_position.z >= 15.0
    {
        return None;
    }

    let confidence = flip_reset_confidence(&features);
    (confidence >= 0.52).then_some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
    })
}
