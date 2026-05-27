use super::*;

pub(crate) fn flip_reset_confidence(features: &FlipResetTouchFeatures) -> f32 {
    let below_car_score = (-features.local_ball_position.z / 180.0).clamp(0.0, 1.0);
    let alignment_score = ((features.underside_alignment - 0.45) / 0.50).clamp(0.0, 1.0);
    let touch_score = (1.0 - ((features.scaled_touch_distance - 20.0) / 220.0)).clamp(0.0, 1.0);
    let height_score = ((features.player_position.z - 70.0) / 500.0).clamp(0.0, 1.0);
    let footprint_score = (1.0
        - (features.local_ball_position.x.abs() / 260.0).clamp(0.0, 1.0) * 0.5
        - (features.local_ball_position.y.abs() / 260.0).clamp(0.0, 1.0) * 0.5)
        .clamp(0.0, 1.0);
    0.28 * below_car_score
        + 0.26 * alignment_score
        + 0.20 * touch_score
        + 0.14 * height_score
        + 0.12 * footprint_score
}
