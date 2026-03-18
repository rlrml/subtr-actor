use crate::*;
use boxcars;
use std::sync::Arc;

fn or_zero_boxcars_3f() -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

type RigidBodyArrayResult<F> = SubtrActorResult<[F; 12]>;
type RigidBodyQuaternionArrayResult<F> = SubtrActorResult<[F; 13]>;
type RigidBodyBasisArrayResult<F> = SubtrActorResult<[F; 15]>;

/// Converts a rigid body into position, Euler rotation, and velocity features.
pub fn get_rigid_body_properties<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> RigidBodyArrayResult<F>
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
    let (rx, ry, rz) =
        glam::quat(rotation.x, rotation.y, rotation.z, rotation.w).to_euler(glam::EulerRot::XYZ);
    convert_all_floats!(
        location.x,
        location.y,
        location.z,
        rx,
        ry,
        rz,
        linear_velocity.x,
        linear_velocity.y,
        linear_velocity.z,
        angular_velocity.x,
        angular_velocity.y,
        angular_velocity.z,
    )
}

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

fn default_rb_state<F: TryFrom<f32>>() -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        convert_float_conversion_error,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )
}

fn default_rb_state_quaternion<F: TryFrom<f32>>() -> RigidBodyQuaternionArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

fn default_rb_state_basis<F: TryFrom<f32>>() -> RigidBodyBasisArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

fn default_rb_state_no_velocities<F: TryFrom<f32>>() -> SubtrActorResult<[F; 7]>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

build_global_feature_adder!(
    SecondsRemaining,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        convert_all_floats!(processor.get_seconds_remaining().unwrap_or(0) as f32)
    },
    "seconds remaining"
);

build_global_feature_adder!(
    CurrentTime,
    |_, _processor, _frame, _index, current_time: f32| { convert_all_floats!(current_time) },
    "current time"
);

build_global_feature_adder!(
    ReplicatedStateName,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        convert_all_floats!(processor.get_replicated_state_name().unwrap_or(0) as f32)
    },
    "game state"
);

build_global_feature_adder!(
    ReplicatedGameStateTimeRemaining,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        convert_all_floats!(processor
            .get_replicated_game_state_time_remaining()
            .unwrap_or(0) as f32)
    },
    "kickoff countdown"
);

build_global_feature_adder!(
    BallHasBeenHit,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        convert_all_floats!(if processor.get_ball_has_been_hit().unwrap_or(false) {
            1.0
        } else {
            0.0
        })
    },
    "ball has been hit"
);

build_global_feature_adder!(
    FrameTime,
    |_, _processor, frame: &boxcars::Frame, _index, _current_time| {
        convert_all_floats!(frame.time)
    },
    "frame time"
);

build_global_feature_adder!(
    BallRigidBody,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        processor
            .get_normalized_ball_rigid_body()
            .and_then(|rb| get_rigid_body_properties(&rb))
            .or_else(|_| default_rb_state())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - linear velocity x",
    "Ball - linear velocity y",
    "Ball - linear velocity z",
    "Ball - angular velocity x",
    "Ball - angular velocity y",
    "Ball - angular velocity z",
);

build_global_feature_adder!(
    BallRigidBodyNoVelocities,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        processor
            .get_normalized_ball_rigid_body()
            .and_then(|rb| get_rigid_body_properties_no_velocities(&rb))
            .or_else(|_| default_rb_state_no_velocities())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

build_global_feature_adder!(
    VelocityAddedBallRigidBodyNoVelocities,
    |_, processor: &ReplayProcessor, _frame, _index, current_time: f32| {
        processor
            .get_velocity_applied_ball_rigid_body(current_time)
            .and_then(|rb| get_rigid_body_properties_no_velocities(&rb))
            .or_else(|_| default_rb_state_no_velocities())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

/// Global feature adder that samples an interpolated ball rigid body.
#[derive(derive_new::new)]
pub struct InterpolatedBallRigidBodyNoVelocities<F> {
    close_enough_to_frame_time: f32,
    _zero: std::marker::PhantomData<F>,
}

impl<F> InterpolatedBallRigidBodyNoVelocities<F> {
    /// Creates the feature adder with the tolerated snap-to-frame threshold.
    pub fn arc_new(close_enough_to_frame_time: f32) -> Arc<Self> {
        Arc::new(Self::new(close_enough_to_frame_time))
    }
}

global_feature_adder!(
    InterpolatedBallRigidBodyNoVelocities,
    |s: &InterpolatedBallRigidBodyNoVelocities<F>,
     processor: &ReplayProcessor,
     _frame: &boxcars::Frame,
     _index,
     current_time: f32| {
        processor
            .get_interpolated_ball_rigid_body(current_time, s.close_enough_to_frame_time)
            .map(|v| get_rigid_body_properties_no_velocities(&v))
            .unwrap_or_else(|_| default_rb_state_no_velocities())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

build_player_feature_adder!(
    PlayerRigidBody,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_normalized_player_rigid_body(player_id) {
            get_rigid_body_properties(&rb)
        } else {
            default_rb_state()
        }
    },
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "linear velocity x",
    "linear velocity y",
    "linear velocity z",
    "angular velocity x",
    "angular velocity y",
    "angular velocity z",
);

build_player_feature_adder!(
    PlayerRigidBodyNoVelocities,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_normalized_player_rigid_body(player_id) {
            get_rigid_body_properties_no_velocities(&rb)
        } else {
            default_rb_state_no_velocities()
        }
    },
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "rotation w"
);

build_player_feature_adder!(
    VelocityAddedPlayerRigidBodyNoVelocities,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, current_time: f32| {
        if let Ok(rb) = processor.get_velocity_applied_player_rigid_body(player_id, current_time) {
            get_rigid_body_properties_no_velocities(&rb)
        } else {
            default_rb_state_no_velocities()
        }
    },
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "rotation w"
);

/// Per-player feature adder that samples an interpolated car rigid body.
#[derive(derive_new::new)]
pub struct InterpolatedPlayerRigidBodyNoVelocities<F> {
    close_enough_to_frame_time: f32,
    _zero: std::marker::PhantomData<F>,
}

impl<F> InterpolatedPlayerRigidBodyNoVelocities<F> {
    /// Creates the feature adder with the tolerated snap-to-frame threshold.
    pub fn arc_new(close_enough_to_frame_time: f32) -> Arc<Self> {
        Arc::new(Self::new(close_enough_to_frame_time))
    }
}

player_feature_adder!(
    InterpolatedPlayerRigidBodyNoVelocities,
    |s: &InterpolatedPlayerRigidBodyNoVelocities<F>,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame: &boxcars::Frame,
     _index,
     current_time: f32| {
        processor
            .get_interpolated_player_rigid_body(
                player_id,
                current_time,
                s.close_enough_to_frame_time,
            )
            .map(|v| get_rigid_body_properties_no_velocities(&v))
            .unwrap_or_else(|_| default_rb_state_no_velocities())
    },
    "i position x",
    "i position y",
    "i position z",
    "i rotation x",
    "i rotation y",
    "i rotation z",
    "i rotation w"
);

build_player_feature_adder!(
    PlayerRelativeBallPosition,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        let relative_position = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                vec_to_glam(&ball_rigid_body.location) - vec_to_glam(&player_rigid_body.location)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            relative_position.x,
            relative_position.y,
            relative_position.z
        )
    },
    "relative ball position x",
    "relative ball position y",
    "relative ball position z"
);

build_player_feature_adder!(
    PlayerRelativeBallVelocity,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        let relative_velocity = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                vec_to_glam(
                    &ball_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                ) - vec_to_glam(
                    &player_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                )
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            relative_velocity.x,
            relative_velocity.y,
            relative_velocity.z
        )
    },
    "relative ball velocity x",
    "relative ball velocity y",
    "relative ball velocity z"
);

build_player_feature_adder!(
    PlayerLocalRelativeBallPosition,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        let local_relative_position = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                let player_rotation = player_rigid_body.rotation;
                let player_quat = glam::quat(
                    player_rotation.x,
                    player_rotation.y,
                    player_rotation.z,
                    player_rotation.w,
                );
                let world_relative_position = vec_to_glam(&ball_rigid_body.location)
                    - vec_to_glam(&player_rigid_body.location);
                player_quat.inverse().mul_vec3(world_relative_position)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            local_relative_position.x,
            local_relative_position.y,
            local_relative_position.z
        )
    },
    "local relative ball position x",
    "local relative ball position y",
    "local relative ball position z"
);

build_player_feature_adder!(
    PlayerLocalRelativeBallVelocity,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        let local_relative_velocity = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                let player_rotation = player_rigid_body.rotation;
                let player_quat = glam::quat(
                    player_rotation.x,
                    player_rotation.y,
                    player_rotation.z,
                    player_rotation.w,
                );
                let world_relative_velocity = vec_to_glam(
                    &ball_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                ) - vec_to_glam(
                    &player_rigid_body
                        .linear_velocity
                        .unwrap_or_else(or_zero_boxcars_3f),
                );
                player_quat.inverse().mul_vec3(world_relative_velocity)
            })
            .unwrap_or(glam::f32::Vec3::ZERO);
        convert_all_floats!(
            local_relative_velocity.x,
            local_relative_velocity.y,
            local_relative_velocity.z
        )
    },
    "local relative ball velocity x",
    "local relative ball velocity y",
    "local relative ball velocity z"
);

build_player_feature_adder!(
    PlayerBallDistance,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        let distance = processor
            .get_normalized_player_rigid_body(player_id)
            .ok()
            .zip(processor.get_normalized_ball_rigid_body().ok())
            .map(|(player_rigid_body, ball_rigid_body)| {
                (vec_to_glam(&player_rigid_body.location) - vec_to_glam(&ball_rigid_body.location))
                    .length()
            })
            .unwrap_or(0.0);
        convert_all_floats!(distance)
    },
    "distance to ball"
);

build_player_feature_adder!(
    PlayerBoost,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        convert_all_floats!(processor.get_player_boost_level(player_id).unwrap_or(0.0))
    },
    "boost level (raw replay units)"
);

fn u8_get_f32(v: u8) -> SubtrActorResult<f32> {
    Ok(v.into())
}

build_player_feature_adder!(
    PlayerJump,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     _frame_number,
     _current_time: f32| {
        convert_all_floats!(
            processor
                .get_dodge_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_double_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
        )
    },
    "dodge active",
    "jump active",
    "double jump active"
);

build_player_feature_adder!(
    PlayerAnyJump,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     _frame_number,
     _current_time: f32| {
        let dodge_is_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2;
        let jump_is_active = processor.get_jump_active(player_id).unwrap_or(0) % 2;
        let double_jump_is_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2;
        let value: f32 = [dodge_is_active, jump_is_active, double_jump_is_active]
            .into_iter()
            .enumerate()
            .map(|(index, is_active)| (1 << index) * is_active)
            .sum::<u8>() as f32;
        convert_all_floats!(value)
    },
    "any_jump_active"
);

build_player_feature_adder!(
    PlayerDodgeRefreshed,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     _frame_number,
     _current_time: f32| {
        let dodge_refresh_count = processor
            .current_frame_dodge_refreshed_events()
            .iter()
            .filter(|event| &event.player == player_id)
            .count() as f32;
        convert_all_floats!(dodge_refresh_count)
    },
    "dodge refresh count"
);

const DEMOLISH_APPEARANCE_FRAME_COUNT: usize = 30;

build_player_feature_adder!(
    PlayerDemolishedBy,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     frame_number,
     _current_time: f32| {
        let demolisher_index = processor
            .demolishes
            .iter()
            .find(|demolish_info| {
                &demolish_info.victim == player_id
                    && frame_number - demolish_info.frame < DEMOLISH_APPEARANCE_FRAME_COUNT
            })
            .map(|demolish_info| {
                processor
                    .iter_player_ids_in_order()
                    .position(|player_id| player_id == &demolish_info.attacker)
                    .unwrap_or_else(|| processor.iter_player_ids_in_order().count())
            })
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(-1);
        convert_all_floats!(demolisher_index as f32)
    },
    "player demolished by"
);

build_player_feature_adder!(
    PlayerRigidBodyQuaternions,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_normalized_player_rigid_body(player_id) {
            let rotation = rb.rotation;
            let location = rb.location;
            convert_all_floats!(
                location.x, location.y, location.z, rotation.x, rotation.y, rotation.z, rotation.w
            )
        } else {
            convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0)
        }
    },
    "position x",
    "position y",
    "position z",
    "quaternion x",
    "quaternion y",
    "quaternion z",
    "quaternion w"
);

build_player_feature_adder!(
    PlayerRigidBodyQuaternionVelocities,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_normalized_player_rigid_body(player_id) {
            get_rigid_body_properties_quaternion(&rb)
        } else {
            default_rb_state_quaternion()
        }
    },
    "position x",
    "position y",
    "position z",
    "quaternion x",
    "quaternion y",
    "quaternion z",
    "quaternion w",
    "linear velocity x",
    "linear velocity y",
    "linear velocity z",
    "angular velocity x",
    "angular velocity y",
    "angular velocity z",
);

build_player_feature_adder!(
    PlayerRigidBodyBasis,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_normalized_player_rigid_body(player_id) {
            get_rigid_body_properties_basis(&rb)
        } else {
            default_rb_state_basis()
        }
    },
    "position x",
    "position y",
    "position z",
    "forward x",
    "forward y",
    "forward z",
    "up x",
    "up y",
    "up z",
    "linear velocity x",
    "linear velocity y",
    "linear velocity z",
    "angular velocity x",
    "angular velocity y",
    "angular velocity z",
);

build_global_feature_adder!(
    BallRigidBodyQuaternions,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        match processor.get_normalized_ball_rigid_body() {
            Ok(rb) => {
                let rotation = rb.rotation;
                let location = rb.location;
                convert_all_floats!(
                    location.x, location.y, location.z, rotation.x, rotation.y, rotation.z,
                    rotation.w
                )
            }
            Err(_) => convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        }
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - quaternion x",
    "Ball - quaternion y",
    "Ball - quaternion z",
    "Ball - quaternion w"
);

build_global_feature_adder!(
    BallRigidBodyQuaternionVelocities,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        processor
            .get_normalized_ball_rigid_body()
            .and_then(|rb| get_rigid_body_properties_quaternion(&rb))
            .or_else(|_| default_rb_state_quaternion())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - quaternion x",
    "Ball - quaternion y",
    "Ball - quaternion z",
    "Ball - quaternion w",
    "Ball - linear velocity x",
    "Ball - linear velocity y",
    "Ball - linear velocity z",
    "Ball - angular velocity x",
    "Ball - angular velocity y",
    "Ball - angular velocity z",
);

build_global_feature_adder!(
    BallRigidBodyBasis,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        processor
            .get_normalized_ball_rigid_body()
            .and_then(|rb| get_rigid_body_properties_basis(&rb))
            .or_else(|_| default_rb_state_basis())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - forward x",
    "Ball - forward y",
    "Ball - forward z",
    "Ball - up x",
    "Ball - up y",
    "Ball - up z",
    "Ball - linear velocity x",
    "Ball - linear velocity y",
    "Ball - linear velocity z",
    "Ball - angular velocity x",
    "Ball - angular velocity y",
    "Ball - angular velocity z",
);
