use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FlipResetHeuristic {
    pub confidence: f32,
    pub local_ball_position: glam::Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FlipResetTouchFeatures {
    pub player_position: glam::Vec3,
    pub ball_position: glam::Vec3,
    pub center_distance: f32,
    pub local_ball_position: glam::Vec3,
    pub scaled_touch_distance: f32,
    pub underside_alignment: f32,
}

pub(crate) fn scale_factor_for_positions(
    ball_position: glam::Vec3,
    player_position: glam::Vec3,
) -> f32 {
    if ball_position
        .truncate()
        .abs()
        .max(player_position.truncate().abs())
        .max_element()
        < 200.0
    {
        100.0
    } else {
        1.0
    }
}

pub(crate) fn build_touch_features(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetTouchFeatures> {
    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);
    let ball_position = raw_ball_position * scale_factor;
    let player_position = raw_player_position * scale_factor;
    let relative_ball_position = ball_position - player_position;
    let center_distance = relative_ball_position.length();
    if !center_distance.is_finite() || center_distance <= 30.0 || center_distance >= 550.0 {
        return None;
    }

    let player_rotation = quat_to_glam(&player_body.rotation);
    let local_ball_position = player_rotation.inverse() * relative_ball_position;
    let car_up = (player_rotation * glam::Vec3::Z).normalize_or_zero();
    let underside_alignment = (-car_up).dot(relative_ball_position.normalize_or_zero());
    let scaled_touch_distance = closest_approach_distance * scale_factor;
    Some(FlipResetTouchFeatures {
        player_position,
        ball_position,
        center_distance,
        local_ball_position,
        scaled_touch_distance,
        underside_alignment,
    })
}
