use super::{glam_to_quat, glam_to_vec, quat_to_glam, vec_to_glam};
use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

/// Interpolates between two [`boxcars::RigidBody`] states based on the provided time.
pub fn get_interpolated_rigid_body(
    start_body: &boxcars::RigidBody,
    start_time: f32,
    end_body: &boxcars::RigidBody,
    end_time: f32,
    time: f32,
) -> SubtrActorResult<boxcars::RigidBody> {
    if !(start_time <= time && time <= end_time) {
        return SubtrActorError::new_result(SubtrActorErrorVariant::InterpolationTimeOrderError {
            start_time,
            time,
            end_time,
        });
    }

    let interpolation_amount = (time - start_time) / (end_time - start_time);
    let start_position = vec_to_glam(&start_body.location);
    let end_position = vec_to_glam(&end_body.location);
    let interpolated_location = start_position.lerp(end_position, interpolation_amount);
    let start_rotation = quat_to_glam(&start_body.rotation);
    let end_rotation = quat_to_glam(&end_body.rotation);
    let interpolated_rotation = start_rotation.slerp(end_rotation, interpolation_amount);

    Ok(boxcars::RigidBody {
        location: glam_to_vec(&interpolated_location),
        rotation: glam_to_quat(&interpolated_rotation),
        sleeping: start_body.sleeping,
        linear_velocity: start_body.linear_velocity,
        angular_velocity: start_body.angular_velocity,
    })
}

#[cfg(test)]
#[path = "geometry_interpolation_tests.rs"]
mod tests;
