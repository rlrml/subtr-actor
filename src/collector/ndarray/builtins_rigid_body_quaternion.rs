use super::*;

/// Converts a rigid body into position, quaternion rotation, and velocity features.
pub fn get_rigid_body_properties_quaternion<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> RigidBodyQuaternionArrayResult<F>
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
    convert_all_floats!(
        location.x,
        location.y,
        location.z,
        rotation.x,
        rotation.y,
        rotation.z,
        rotation.w,
        linear_velocity.x,
        linear_velocity.y,
        linear_velocity.z,
        angular_velocity.x,
        angular_velocity.y,
        angular_velocity.z,
    )
}
