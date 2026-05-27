use super::*;

build_player_feature_adder!(
    PlayerRigidBody,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
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
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
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
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, current_time: f32| {
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
     processor: &dyn ProcessorView,
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
