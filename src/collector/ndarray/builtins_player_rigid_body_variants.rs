use super::*;

build_player_feature_adder!(
    PlayerRigidBodyQuaternions,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
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
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
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
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
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
