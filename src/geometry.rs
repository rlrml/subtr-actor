use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

pub fn vec_to_glam(v: &boxcars::Vector3f) -> glam::f32::Vec3 {
    glam::f32::Vec3::new(v.x, v.y, v.z)
}

pub fn glam_to_vec(v: &glam::f32::Vec3) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

pub fn quat_to_glam(q: &boxcars::Quaternion) -> glam::Quat {
    glam::Quat::from_xyzw(q.x, q.y, q.z, q.w)
}

pub fn glam_to_quat(rotation: &glam::Quat) -> boxcars::Quaternion {
    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

pub fn apply_velocities_to_rigid_body(
    rigid_body: &boxcars::RigidBody,
    time_delta: f32,
) -> boxcars::RigidBody {
    let mut interpolated = *rigid_body;
    if time_delta == 0.0 {
        return interpolated;
    }
    let linear_velocity = interpolated.linear_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let location = vec_to_glam(&rigid_body.location) + (time_delta * vec_to_glam(&linear_velocity));
    interpolated.location = glam_to_vec(&location);
    interpolated.rotation = apply_angular_velocity(rigid_body, time_delta);
    interpolated
}

/// Ranks how plausible it is that `player_body` was the car that touched the
/// ball near the current frame, using constant-velocity closest approach.
///
/// The frame's ball state can already be slightly post-contact, so we do not
/// just compare current distance. Instead we look for the minimum ball/car
/// separation over a short window centered slightly before the frame time.
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

    let relative_velocity =
        vec_to_glam(&player_body.linear_velocity.unwrap_or(boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })) - vec_to_glam(&ball_body.linear_velocity.unwrap_or(boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }));
    let relative_speed_squared = relative_velocity.length_squared();
    let closest_time = if relative_speed_squared > f32::EPSILON {
        (-relative_position.dot(relative_velocity) / relative_speed_squared)
            .clamp(-TOUCH_LOOKBACK_SECONDS, TOUCH_LOOKAHEAD_SECONDS)
    } else {
        0.0
    };
    let closest_distance = (relative_position + relative_velocity * closest_time).length();
    if !closest_distance.is_finite() {
        return None;
    }

    Some((closest_distance, current_distance))
}

fn apply_angular_velocity(rigid_body: &boxcars::RigidBody, time_delta: f32) -> boxcars::Quaternion {
    // XXX: This approach seems to give some unexpected results. There may be a
    // unit mismatch or some other type of issue.
    let rbav = rigid_body.angular_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let angular_velocity = glam::Vec3::new(rbav.x, rbav.y, rbav.z);
    let magnitude = angular_velocity.length();
    let angular_velocity_unit_vector = angular_velocity.normalize_or_zero();

    let mut rotation = glam::Quat::from_xyzw(
        rigid_body.rotation.x,
        rigid_body.rotation.y,
        rigid_body.rotation.z,
        rigid_body.rotation.w,
    );

    if angular_velocity_unit_vector.length() != 0.0 {
        let delta_rotation =
            glam::Quat::from_axis_angle(angular_velocity_unit_vector, magnitude * time_delta);
        rotation *= delta_rotation;
    }

    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

/// Interpolates between two [`boxcars::RigidBody`] states based on the provided time.
///
/// # Arguments
///
/// * `start_body` - The initial `RigidBody` state.
/// * `start_time` - The timestamp of the initial `RigidBody` state.
/// * `end_body` - The final `RigidBody` state.
/// * `end_time` - The timestamp of the final `RigidBody` state.
/// * `time` - The desired timestamp to interpolate to.
///
/// # Returns
///
/// A new [`boxcars::RigidBody`] that represents the interpolated state at the specified time.
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

    let duration = end_time - start_time;
    let interpolation_amount = (time - start_time) / duration;
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
