use super::*;

build_global_feature_adder!(
    BallRigidBody,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
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
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
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
    |_, processor: &dyn ProcessorView, _frame, _index, current_time: f32| {
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
     processor: &dyn ProcessorView,
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
