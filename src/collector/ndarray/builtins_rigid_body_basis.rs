use super::*;

/// Converts a rigid body into position, basis vectors, and velocity features.
pub fn get_rigid_body_properties_basis<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> RigidBodyBasisArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let linear_velocity = rigid_body
        .linear_velocity
        .unwrap_or_else(or_zero_boxcars_3f);
    let angular_velocity = rigid_body
        .angular_velocity
        .unwrap_or_else(or_zero_boxcars_3f);
    let rotation = rigid_body.rotation;
    let location = rigid_body.location;
    let quat = glam::quat(rotation.x, rotation.y, rotation.z, rotation.w);
    let forward = quat.mul_vec3(glam::Vec3::X);
    let up = quat.mul_vec3(glam::Vec3::Z);
    convert_all_floats!(
        location.x,
        location.y,
        location.z,
        forward.x,
        forward.y,
        forward.z,
        up.x,
        up.y,
        up.z,
        linear_velocity.x,
        linear_velocity.y,
        linear_velocity.z,
        angular_velocity.x,
        angular_velocity.y,
        angular_velocity.z,
    )
}
