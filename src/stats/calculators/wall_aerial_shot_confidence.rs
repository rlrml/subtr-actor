use super::*;

pub(crate) fn wall_aerial_shot_confidence(
    time_since_takeoff: f32,
    player_position: glam::Vec3,
    ball_speed: Option<f32>,
    goal_alignment: Option<f32>,
) -> f32 {
    let confidence = 0.42
        + 0.20
            * (1.0
                - wall_aerial_normalize_score(
                    time_since_takeoff,
                    0.15,
                    WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS,
                ))
        + 0.16
            * wall_aerial_normalize_score(player_position.z, WALL_AERIAL_MIN_TOUCH_PLAYER_Z, 850.0)
        + 0.12 * goal_alignment.unwrap_or(0.0).clamp(0.0, 1.0)
        + 0.10 * wall_aerial_normalize_score(ball_speed.unwrap_or(0.0), 600.0, 1800.0);

    confidence.clamp(0.0, 1.0)
}
