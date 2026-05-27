use super::*;

/// Converts a rigid body into position and quaternion-rotation features only.
pub fn get_rigid_body_properties_no_velocities<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> SubtrActorResult<[F; 7]>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let rotation = rigid_body.rotation;
    let location = rigid_body.location;
    convert_all_floats!(
        location.x, location.y, location.z, rotation.x, rotation.y, rotation.z, rotation.w
    )
}
