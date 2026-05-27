use super::*;

/// Represents a player's state for a single frame in a Rocket League replay.
///
/// Contains comprehensive information about a player's position, movement,
/// and control inputs during a specific frame of the replay.
///
/// # Variants
///
/// - [`Empty`](PlayerFrame::Empty) - Indicates the player state is unavailable
/// - [`Data`](PlayerFrame::Data) - Contains the player's complete state information
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum PlayerFrame {
    /// Empty frame indicating the player state is unavailable
    Empty,
    /// Frame containing the player's complete state data
    Data {
        /// The player's rigid body containing position, rotation, and velocity information
        #[ts(as = "crate::ts_bindings::RigidBodyTs")]
        rigid_body: boxcars::RigidBody,
        /// The player's current boost amount in raw replay units (0.0 to 255.0)
        boost_amount: f32,
        /// Whether the player is actively using boost
        boost_active: bool,
        /// Whether the player is actively powersliding / holding handbrake
        powerslide_active: bool,
        /// Whether the player is actively jumping
        jump_active: bool,
        /// Whether the player is performing a double jump
        double_jump_active: bool,
        /// Whether the player is performing a dodge maneuver
        dodge_active: bool,
        /// The player's name as it appears in the replay
        player_name: Option<String>,
        /// The team the player belongs to (0 or 1)
        team: Option<i32>,
        /// Whether the player is on team 0 (blue team typically)
        is_team_0: Option<bool>,
    },
}
