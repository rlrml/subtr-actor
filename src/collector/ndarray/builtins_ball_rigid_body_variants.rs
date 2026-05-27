use super::*;

build_global_feature_adder!(
    BallRigidBodyQuaternions,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
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
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
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
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
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
