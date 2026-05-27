use super::*;

pub(crate) fn flip_reset_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetHeuristic> {
    let features = build_touch_features(ball_body, player_body, closest_approach_distance)?;
    if features.player_position.z < 95.0 || features.ball_position.z < 80.0 {
        return None;
    }
    if features.scaled_touch_distance > 220.0
        || features.underside_alignment < 0.60
        || features.local_ball_position.x.abs() > 240.0
        || features.local_ball_position.y.abs() > 240.0
        || features.local_ball_position.z >= 10.0
    {
        return None;
    }

    let confidence = flip_reset_confidence(&features);
    (confidence >= 0.55).then_some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
    })
}

pub(crate) fn flip_reset_followup_touch_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetHeuristic> {
    let features = build_touch_features(ball_body, player_body, closest_approach_distance)?;
    let confidence = flip_reset_confidence(&features);
    if confidence < 0.45
        || features.local_ball_position.z >= 20.0
        || features.underside_alignment < 0.25
    {
        return None;
    }

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
    })
}

#[cfg(test)]
#[path = "flip_reset_candidates_tests.rs"]
mod tests;
