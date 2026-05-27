use super::vec_to_glam;

/// Ranks how plausible it is that `player_body` was the car that touched the
/// ball near the current frame, using constant-velocity closest approach.
pub(crate) fn touch_candidate_rank(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
) -> Option<(f32, f32)> {
    const TOUCH_LOOKBACK_SECONDS: f32 = 0.12;
    const TOUCH_LOOKAHEAD_SECONDS: f32 = 0.03;

    let relative_position = vec_to_glam(&player_body.location) - vec_to_glam(&ball_body.location);
    let current_distance = relative_position.length();
    if !current_distance.is_finite() {
        return None;
    }

    let relative_velocity = player_velocity(player_body) - player_velocity(ball_body);
    let relative_speed_squared = relative_velocity.length_squared();
    let closest_time = if relative_speed_squared > f32::EPSILON {
        (-relative_position.dot(relative_velocity) / relative_speed_squared)
            .clamp(-TOUCH_LOOKBACK_SECONDS, TOUCH_LOOKAHEAD_SECONDS)
    } else {
        0.0
    };
    let closest_distance = (relative_position + relative_velocity * closest_time).length();
    closest_distance
        .is_finite()
        .then_some((closest_distance, current_distance))
}

fn player_velocity(body: &boxcars::RigidBody) -> glam::Vec3 {
    vec_to_glam(&body.linear_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }))
}

#[cfg(test)]
#[path = "geometry_touch_tests.rs"]
mod tests;
