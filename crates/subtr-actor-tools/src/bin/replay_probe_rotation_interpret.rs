use super::rotation_types::{EulerMode, QuaternionMode};

pub(crate) fn reinterpret_quaternion(
    raw: boxcars::Quaternion,
    mode: QuaternionMode,
) -> Option<glam::Quat> {
    let source = [raw.x, raw.y, raw.z];
    let values = [
        source[mode.order[0]] * f32::from(mode.signs[0]),
        source[mode.order[1]] * f32::from(mode.signs[1]),
        source[mode.order[2]] * f32::from(mode.signs[2]),
    ];
    let mut components = [0.0; 4];
    let mut value_index = 0;
    for (slot, component) in components.iter_mut().enumerate() {
        if slot == mode.missing_slot {
            continue;
        }
        *component = values[value_index];
        value_index += 1;
    }
    if mode.reconstruct_missing {
        let sum_squares: f32 = components
            .iter()
            .map(|component| component * component)
            .sum();
        if sum_squares > 1.0 + 0.001 {
            return None;
        }
        components[mode.missing_slot] = (1.0 - sum_squares.min(1.0)).sqrt();
    }
    let quaternion =
        glam::Quat::from_xyzw(components[0], components[1], components[2], components[3]);
    (quaternion.length_squared() > f32::EPSILON).then(|| quaternion.normalize())
}

pub(crate) fn reinterpret_euler_rotation(raw: boxcars::Quaternion, mode: EulerMode) -> glam::Quat {
    let source = [raw.x, raw.y, raw.z];
    let factor = mode.scale.factor();
    let values = [
        source[mode.order[0]] * f32::from(mode.signs[0]) * factor,
        source[mode.order[1]] * f32::from(mode.signs[1]) * factor,
        source[mode.order[2]] * f32::from(mode.signs[2]) * factor,
    ];
    glam::Quat::from_euler(
        mode.rotation_order.to_glam(),
        values[0],
        values[1],
        values[2],
    )
}

pub(crate) fn rotation_alignment(
    quaternion: glam::Quat,
    linear_velocity: boxcars::Vector3f,
) -> Option<(f32, f32)> {
    let forward = quaternion * glam::Vec3::X;
    let forward_xy = forward.truncate().normalize_or_zero();
    let velocity_xy = glam::Vec2::new(linear_velocity.x, linear_velocity.y).normalize_or_zero();
    let alignment = forward_xy.dot(velocity_xy);
    alignment
        .is_finite()
        .then_some((alignment, (quaternion * glam::Vec3::Z).z))
}

pub(crate) fn derive_world_angular_velocity(
    previous_rotation: glam::Quat,
    mut current_rotation: glam::Quat,
    dt: f32,
) -> Option<glam::Vec3> {
    if dt <= 0.0 {
        return None;
    }
    if previous_rotation.dot(current_rotation) < 0.0 {
        current_rotation = glam::Quat::from_xyzw(
            -current_rotation.x,
            -current_rotation.y,
            -current_rotation.z,
            -current_rotation.w,
        );
    }
    let delta = current_rotation * previous_rotation.inverse();
    let (axis, angle) = delta.to_axis_angle();
    let angular_velocity = axis * (angle / dt);
    angular_velocity.is_finite().then_some(angular_velocity)
}
