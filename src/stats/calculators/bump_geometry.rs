use super::*;

pub(super) fn vec3_to_array(v: glam::Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn horizontal(v: glam::Vec3) -> glam::Vec2 {
    glam::Vec2::new(v.x, v.y)
}

pub(super) fn swept_horizontal_distance(
    left_previous: glam::Vec3,
    left_current: glam::Vec3,
    right_previous: glam::Vec3,
    right_current: glam::Vec3,
) -> f32 {
    let relative_start = horizontal(left_previous - right_previous);
    let relative_delta =
        horizontal((left_current - left_previous) - (right_current - right_previous));
    let closest_t = if relative_delta.length_squared() > f32::EPSILON {
        (-relative_start.dot(relative_delta) / relative_delta.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (relative_start + relative_delta * closest_t).length()
}

pub(super) fn contact_normal(
    left_previous: glam::Vec3,
    left_current: glam::Vec3,
    right_previous: glam::Vec3,
    right_current: glam::Vec3,
) -> Option<glam::Vec3> {
    let relative_current = right_current - left_current;
    let current_horizontal = glam::Vec3::new(relative_current.x, relative_current.y, 0.0);
    if current_horizontal.length_squared() > 1.0 {
        return Some(current_horizontal.normalize());
    }

    let relative_previous = right_previous - left_previous;
    let previous_horizontal = glam::Vec3::new(relative_previous.x, relative_previous.y, 0.0);
    (previous_horizontal.length_squared() > 1.0).then(|| previous_horizontal.normalize())
}

pub(super) fn directional_candidate(
    initiator_previous: &boxcars::RigidBody,
    initiator_current: &boxcars::RigidBody,
    victim_previous: &boxcars::RigidBody,
    victim_current: &boxcars::RigidBody,
    normal: glam::Vec3,
) -> Option<DirectionalBumpCandidate> {
    let initiator_previous_velocity = rigid_body_velocity(initiator_previous);
    let initiator_current_velocity = rigid_body_velocity(initiator_current);
    let victim_previous_velocity = rigid_body_velocity(victim_previous);
    let victim_current_velocity = rigid_body_velocity(victim_current);

    let closing_speed = (initiator_previous_velocity - victim_previous_velocity).dot(normal);
    let victim_impulse = (victim_current_velocity - victim_previous_velocity).dot(normal);
    let initiator_slowdown = (initiator_previous_velocity - initiator_current_velocity).dot(normal);
    let speed_advantage =
        initiator_previous_velocity.dot(normal) - victim_previous_velocity.dot(normal);
    let forward_alignment = (quat_to_glam(&initiator_previous.rotation) * glam::Vec3::X)
        .dot(normal)
        .max(0.0);

    if !closing_speed.is_finite() || !victim_impulse.is_finite() {
        return None;
    }

    Some(DirectionalBumpCandidate {
        score: closing_speed
            + 1.35 * victim_impulse.max(0.0)
            + 0.35 * initiator_slowdown.max(0.0)
            + 220.0 * forward_alignment
            + 0.15 * speed_advantage.max(0.0),
        closing_speed,
        victim_impulse,
        initiator_slowdown,
    })
}

fn rigid_body_velocity(rigid_body: &boxcars::RigidBody) -> glam::Vec3 {
    rigid_body
        .linear_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO)
}
