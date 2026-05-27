use super::*;

/// Represents the ball state for a single frame in a Rocket League replay.
///
/// The ball can either be in an empty state (when ball syncing is disabled or
/// the rigid body is unavailable) or contain full physics data including
/// position, rotation, and velocity information.
///
/// # Variants
///
/// - [`Empty`](BallFrame::Empty) - Indicates the ball is unavailable or ball syncing is disabled
/// - [`Data`](BallFrame::Data) - Contains the ball's rigid body physics information
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum BallFrame {
    /// Empty frame indicating the ball is unavailable or ball syncing is disabled
    Empty,
    /// Frame containing the ball's rigid body physics data
    Data {
        /// The ball's rigid body containing position, rotation, and velocity information
        #[ts(as = "crate::ts_bindings::RigidBodyTs")]
        rigid_body: boxcars::RigidBody,
    },
}

impl BallFrame {
    /// Creates a new [`BallFrame`] from a [`ReplayProcessor`] at the specified time.
    ///
    /// This method extracts the ball's state from the replay processor, handling
    /// cases where ball syncing is disabled or the rigid body is unavailable.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `current_time` - The time in seconds at which to extract the ball state
    ///
    /// # Returns
    ///
    /// Returns a [`BallFrame`] which will be [`Empty`](BallFrame::Empty) if:
    /// - Ball syncing is disabled in the replay
    /// - The ball's rigid body cannot be retrieved
    ///
    /// Otherwise returns [`Data`](BallFrame::Data) containing the ball's rigid body.
    pub(super) fn new_from_processor(processor: &dyn ProcessorView, current_time: f32) -> Self {
        if processor.get_ignore_ball_syncing().unwrap_or(false) {
            Self::Empty
        } else if let Ok(rigid_body) = processor.get_interpolated_ball_rigid_body(current_time, 0.0)
        {
            Self::new_from_rigid_body(rigid_body)
        } else {
            Self::Empty
        }
    }

    /// Creates a new [`BallFrame`] from a rigid body.
    ///
    /// # Arguments
    ///
    /// * `rigid_body` - The ball's rigid body containing physics information
    ///
    /// # Returns
    ///
    /// Returns [`Data`](BallFrame::Data) containing the rigid body even when the
    /// ball is sleeping, so stationary kickoff frames still retain the ball's
    /// position for downstream consumers such as the JS player.
    fn new_from_rigid_body(rigid_body: boxcars::RigidBody) -> Self {
        Self::Data { rigid_body }
    }
}
