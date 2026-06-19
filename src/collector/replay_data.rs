//! # Replay Data Collection Module
//!
//! This module provides comprehensive data structures and collection mechanisms
//! for extracting and organizing Rocket League replay data. It offers a complete
//! representation of ball, player, and game state information across all frames
//! of a replay.
//!
//! The module is built around the [`ReplayDataCollector`] which implements the
//! [`Collector`] trait, allowing it to process replay frames and extract
//! detailed information about player actions, ball movement, and game state.
//!
//! # Key Components
//!
//! - [`ReplayData`] - The complete replay data structure containing all extracted information
//! - [`FrameData`] - Frame-by-frame data including ball, player, and metadata information
//! - [`PlayerFrame`] - Detailed player state including position, controls, and actions
//! - [`BallFrame`] - Ball state including rigid body physics information
//! - [`MetadataFrame`] - Game state metadata including time and score information
//!
//! # Example Usage
//!
//! ```rust
//! use subtr_actor::collector::replay_data::ReplayDataCollector;
//! use boxcars::ParserBuilder;
//!
//! let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
//! let replay = ParserBuilder::new(&data).parse().unwrap();
//!
//! let collector = ReplayDataCollector::new();
//! let replay_data = collector.get_replay_data(&replay).unwrap();
//!
//! // Access frame-by-frame data
//! for metadata_frame in &replay_data.frame_data.metadata_frames {
//!     println!("Time: {:.2}s, Remaining: {}s",
//!              metadata_frame.time, metadata_frame.seconds_remaining);
//! }
//! ```

use boxcars;
use serde::Serialize;

use crate::*;

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
        #[ts(as = "crate::interop::ts_bindings::RigidBodyTs")]
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
    fn new_from_processor(processor: &dyn ProcessorView, current_time: f32) -> Self {
        if processor.get_ignore_ball_syncing().unwrap_or(false) {
            Self::Empty
        } else {
            match processor.get_interpolated_ball_rigid_body(current_time, 0.0) {
                Ok(rigid_body) => Self::new_from_rigid_body(rigid_body),
                _ => Self::Empty,
            }
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

/// Replay-driven continuous camera look state for a player at a single frame.
///
/// Captured from the player's `TAGame.CameraSettingsActor_TA` actor. Rocket
/// League does not replicate the camera's world position, so this is the raw
/// material a renderer uses to *reconstruct* the player's point of view rather
/// than a literal camera transform. The discrete camera toggles (ball cam,
/// behind-view) flip rarely and are carried in the coalesced
/// [`PlayerCameraStateChange`] stream instead of on every frame.
///
/// Every field is optional: it is `None` when the replay does not replicate
/// that attribute for the player (e.g. very old replays, or a player whose
/// camera-settings actor has not appeared yet).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerCameraFrame {
    /// Raw camera pitch byte (0-255) as replicated; convert at display time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<u8>,
    /// Raw camera yaw byte (0-255) as replicated; convert at display time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yaw: Option<u8>,
}

/// Replay-driven vehicle input/state for a player at a single frame.
///
/// Captured from the car's `TAGame.Vehicle_TA` actor and dodge component.
/// These let a renderer drive accurate wheel steering/spin and flip direction
/// instead of estimating them from position deltas. The rarely-flipping driving
/// flag lives in the coalesced [`PlayerCameraStateChange`] stream instead.
///
/// Every field is optional: it is `None` when the replay does not replicate
/// that attribute for the frame.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerInputFrame {
    /// Raw throttle byte (0-255, ~128 neutral); convert at display time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throttle: Option<u8>,
    /// Raw steer byte (0-255, ~128 centered); convert at display time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steer: Option<u8>,
    /// Impulse vector `(x, y, z)` in raw replay units of the most recent
    /// dodge. Meaningful while [`PlayerFrame::Data::dodge_active`] is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dodge_impulse: Option<(f32, f32, f32)>,
    /// Torque vector `(x, y, z)` in raw replay units of the most recent dodge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dodge_torque: Option<(f32, f32, f32)>,
}

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
        #[ts(as = "crate::interop::ts_bindings::RigidBodyTs")]
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
        /// Replay-driven camera state (ball cam, look direction) for the player
        camera: PlayerCameraFrame,
        /// Replay-driven vehicle inputs (throttle, steer, dodge vectors)
        input: PlayerInputFrame,
    },
}

impl PlayerFrame {
    /// Creates a new [`PlayerFrame`] from a [`ReplayProcessor`] for a specific player at the specified time.
    ///
    /// This method extracts comprehensive player state information from the replay processor,
    /// including position, control inputs, and team information.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `player_id` - The unique identifier for the player
    /// * `current_time` - The time in seconds at which to extract the player state
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing a [`PlayerFrame::Data`] value
    /// with the player's complete state information.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - The player's rigid body cannot be retrieved
    fn new_from_processor(
        processor: &dyn ProcessorView,
        player_id: &PlayerId,
        current_time: f32,
    ) -> SubtrActorResult<Self> {
        let rigid_body =
            processor.get_interpolated_player_rigid_body(player_id, current_time, 0.0)?;

        let boost_amount = processor.get_player_boost_level(player_id).unwrap_or(0.0);
        let boost_active = processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1;
        let powerslide_active = processor.get_powerslide_active(player_id).unwrap_or(false);
        let jump_active = processor.get_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let double_jump_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let dodge_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1;

        // Replay-driven continuous camera/vehicle state. Each read is optional:
        // older replays and frames without the attribute simply leave it `None`
        // so consumers can fall back to a synthesized value. Discrete toggles
        // (ball cam, behind-view, driving) are emitted as coalesced
        // `PlayerCameraStateChange`s rather than stored on every frame.
        let camera = PlayerCameraFrame {
            pitch: processor.get_camera_pitch(player_id).ok(),
            yaw: processor.get_camera_yaw(player_id).ok(),
        };
        let input = PlayerInputFrame {
            throttle: processor.get_throttle(player_id).ok(),
            steer: processor.get_steer(player_id).ok(),
            dodge_impulse: processor.get_dodge_impulse(player_id).ok(),
            dodge_torque: processor.get_dodge_torque(player_id).ok(),
        };

        // Extract player identity information
        let player_name = processor.get_player_name(player_id).ok();
        let team = processor
            .get_player_team_key(player_id)
            .ok()
            .and_then(|team_key| team_key.parse::<i32>().ok());
        let is_team_0 = processor.get_player_is_team_0(player_id).ok();

        Ok(Self::from_data(
            rigid_body,
            boost_amount,
            boost_active,
            powerslide_active,
            jump_active,
            double_jump_active,
            dodge_active,
            player_name,
            team,
            is_team_0,
            camera,
            input,
        ))
    }

    /// Creates a [`PlayerFrame`] from the provided data components.
    ///
    /// # Arguments
    ///
    /// * `rigid_body` - The player's rigid body physics information
    /// * `boost_amount` - The player's current boost level in raw replay units (0.0 to 255.0)
    /// * `boost_active` - Whether the player is actively using boost
    /// * `powerslide_active` - Whether the player is actively powersliding
    /// * `jump_active` - Whether the player is actively jumping
    /// * `double_jump_active` - Whether the player is performing a double jump
    /// * `dodge_active` - Whether the player is performing a dodge maneuver
    /// * `player_name` - The player's name, if available
    /// * `team` - The player's team number, if available
    /// * `is_team_0` - Whether the player is on team 0, if available
    /// * `camera` - Replay-driven camera state for the player
    /// * `input` - Replay-driven vehicle input/state for the player
    ///
    /// # Returns
    ///
    /// Returns [`Data`](PlayerFrame::Data) with all provided information, even
    /// when the rigid body is sleeping, so stationary kickoff and reset frames
    /// still retain the player's position for downstream consumers such as the
    /// JS player.
    #[allow(clippy::too_many_arguments)]
    fn from_data(
        rigid_body: boxcars::RigidBody,
        boost_amount: f32,
        boost_active: bool,
        powerslide_active: bool,
        jump_active: bool,
        double_jump_active: bool,
        dodge_active: bool,
        player_name: Option<String>,
        team: Option<i32>,
        is_team_0: Option<bool>,
        camera: PlayerCameraFrame,
        input: PlayerInputFrame,
    ) -> Self {
        Self::Data {
            rigid_body,
            boost_amount,
            boost_active,
            powerslide_active,
            jump_active,
            double_jump_active,
            dodge_active,
            player_name,
            team,
            is_team_0,
            camera,
            input,
        }
    }
}

/// Contains all frame data for a single player throughout the replay.
///
/// This structure holds a chronological sequence of [`PlayerFrame`] instances
/// representing the player's state at each processed frame of the replay.
///
/// # Fields
///
/// * `frames` - A vector of [`PlayerFrame`] instances in chronological order
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerData {
    /// Vector of player frames in chronological order
    frames: Vec<PlayerFrame>,
}

impl PlayerData {
    /// Creates a new empty [`PlayerData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`PlayerData`] with an empty frames vector.
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Adds a player frame at the specified frame index.
    ///
    /// If the frame index is beyond the current length of the frames vector,
    /// empty frames will be inserted to fill the gap before adding the new frame.
    ///
    /// # Arguments
    ///
    /// * `frame_index` - The index at which to insert the frame
    /// * `frame` - The [`PlayerFrame`] to add
    fn add_frame(&mut self, frame_index: usize, frame: PlayerFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(PlayerFrame::Empty)
            }
        }
        self.frames.push(frame)
    }

    /// Returns a reference to the frames vector.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of [`PlayerFrame`] instances.
    pub fn frames(&self) -> &Vec<PlayerFrame> {
        &self.frames
    }

    /// Returns the number of frames in this player's data.
    ///
    /// # Returns
    ///
    /// Returns the total number of frames stored for this player.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// Contains all frame data for the ball throughout the replay.
///
/// This structure holds a chronological sequence of [`BallFrame`] instances
/// representing the ball's state at each processed frame of the replay.
///
/// # Fields
///
/// * `frames` - A vector of [`BallFrame`] instances in chronological order
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallData {
    /// Vector of ball frames in chronological order
    frames: Vec<BallFrame>,
}

impl BallData {
    /// Creates a new empty [`BallData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`BallData`] with an empty frames vector.
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Adds a ball frame at the specified frame index.
    ///
    /// If the frame index is beyond the current length of the frames vector,
    /// empty frames will be inserted to fill the gap before adding the new frame.
    ///
    /// # Arguments
    ///
    /// * `frame_index` - The index at which to insert the frame
    /// * `frame` - The [`BallFrame`] to add
    fn add_frame(&mut self, frame_index: usize, frame: BallFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(BallFrame::Empty)
            }
        }
        self.frames.push(frame)
    }

    /// Returns a reference to the frames vector.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of [`BallFrame`] instances.
    pub fn frames(&self) -> &Vec<BallFrame> {
        &self.frames
    }

    /// Returns the number of frames in the ball data.
    ///
    /// # Returns
    ///
    /// Returns the total number of frames stored for the ball.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// Represents game metadata for a single frame in a Rocket League replay.
///
/// Contains timing information and game state data that applies to the entire
/// game at a specific point in time.
///
/// # Fields
///
/// * `time` - The current time in seconds since the start of the replay
/// * `seconds_remaining` - The number of seconds remaining in the current game period
/// * `replicated_game_state_name` - The game state enum value (indicates countdown, playing, goal, etc.)
/// * `replicated_game_state_time_remaining` - The kickoff countdown timer, usually 3 to 0
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MetadataFrame {
    /// The current time in seconds since the start of the replay
    pub time: f32,
    /// The number of seconds remaining in the current game period
    pub seconds_remaining: i32,
    /// The game state enum value (indicates countdown, playing, goal scored, etc.)
    pub replicated_game_state_name: i32,
    /// The kickoff countdown timer exposed by the replay metadata actor.
    pub replicated_game_state_time_remaining: i32,
}

impl MetadataFrame {
    /// Creates a new [`MetadataFrame`] from a [`ReplayProcessor`] at the specified time.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `time` - The current time in seconds since the start of the replay
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing a [`MetadataFrame`] with the
    /// current time and remaining seconds extracted from the processor.
    ///
    /// # Errors
    ///
    /// Missing replay metadata fields default to 0 so frame export can continue
    /// for replays whose metadata actor does not carry every optional property.
    fn new_from_processor(processor: &dyn ProcessorView, time: f32) -> SubtrActorResult<Self> {
        Ok(Self::new(
            time,
            metadata_i32_or_default(processor.get_seconds_remaining()),
            metadata_i32_or_default(processor.get_replicated_state_name()),
            metadata_i32_or_default(processor.get_replicated_game_state_time_remaining()),
        ))
    }

    /// Creates a new [`MetadataFrame`] with the specified time, seconds remaining, game state,
    /// and kickoff countdown value.
    ///
    /// # Arguments
    ///
    /// * `time` - The current time in seconds since the start of the replay
    /// * `seconds_remaining` - The number of seconds remaining in the current game period
    /// * `replicated_game_state_name` - The game state enum value
    /// * `replicated_game_state_time_remaining` - The kickoff countdown timer
    ///
    /// # Returns
    ///
    /// Returns a new [`MetadataFrame`] with the provided values.
    fn new(
        time: f32,
        seconds_remaining: i32,
        replicated_game_state_name: i32,
        replicated_game_state_time_remaining: i32,
    ) -> Self {
        MetadataFrame {
            time,
            seconds_remaining,
            replicated_game_state_name,
            replicated_game_state_time_remaining,
        }
    }
}

fn metadata_i32_or_default(value: SubtrActorResult<i32>) -> i32 {
    value.unwrap_or(0)
}

#[cfg(test)]
#[path = "replay_data_tests.rs"]
mod replay_data_tests;

/// Contains all frame-by-frame data for a Rocket League replay.
///
/// This structure organizes ball data, player data, and metadata for each
/// frame of the replay, providing a complete picture of the game state
/// throughout the match.
///
/// # Fields
///
/// * `ball_data` - All ball state information across all frames
/// * `players` - Player data for each player, indexed by [`PlayerId`]
/// * `metadata_frames` - Game metadata for each frame including timing information
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FrameData {
    /// All ball state information across all frames
    pub ball_data: BallData,
    /// Player data for each player, indexed by PlayerId
    #[ts(as = "Vec<(crate::interop::ts_bindings::RemoteIdTs, PlayerData)>")]
    pub players: Vec<(PlayerId, PlayerData)>,
    /// Game metadata for each frame including timing information
    pub metadata_frames: Vec<MetadataFrame>,
}

/// Complete replay data structure containing all extracted information from a Rocket League replay.
///
/// This is the top-level structure that contains all processed replay data including
/// frame-by-frame information, replay metadata, and special events like demolitions.
///
/// # Fields
///
/// * `frame_data` - All frame-by-frame data including ball, player, and metadata information
/// * `meta` - Replay metadata including player information, game settings, and statistics
/// * `demolish_infos` - Information about all demolition events that occurred during the replay
/// * `boost_pad_events` - Exact boost pad pickup/availability events detected while processing
/// * `boost_pads` - Resolved standard boost pad layout annotated with replay pad ids when known
/// * `touch_events` - Replay-authored team touch markers; player attribution is derived by stats
/// * `dodge_refreshed_events` - Exact counter-derived dodge refresh events from the replay
/// * `player_stat_events` - Exact shot/save/assist counter increment events
/// * `goal_events` - Exact goal explosion events with scorer and cumulative score when available
/// * `replay_tick_marks` - Replay-authored timeline tick marks/bookmarks
///
/// # Example
///
/// ```rust
/// use subtr_actor::collector::replay_data::ReplayDataCollector;
/// use boxcars::ParserBuilder;
///
/// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
/// let replay = ParserBuilder::new(&data).parse().unwrap();
/// let collector = ReplayDataCollector::new();
/// let replay_data = collector.get_replay_data(&replay).unwrap();
///
/// // Access replay metadata
/// println!("Team 0 players: {}", replay_data.meta.team_zero.len());
///
/// // Access frame data
/// println!("Total frames: {}", replay_data.frame_data.metadata_frames.len());
///
/// // Access demolition events
/// println!("Total demolitions: {}", replay_data.demolish_infos.len());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayData {
    /// All frame-by-frame data including ball, player, and metadata information
    pub frame_data: FrameData,
    /// Replay metadata including player information, game settings, and statistics
    pub meta: ReplayMeta,
    /// Information about all demolition events that occurred during the replay
    pub demolish_infos: Vec<DemolishInfo>,
    /// Exact boost pad pickup and availability events observed during the replay
    pub boost_pad_events: Vec<BoostPadEvent>,
    /// Resolved standard boost pad layout annotated with replay pad ids when known
    pub boost_pads: Vec<ResolvedBoostPad>,
    /// Replay-authored team touch markers observed during the replay
    pub touch_events: Vec<TouchEvent>,
    /// Exact dodge refresh events observed via the replay's refreshed-dodge counter
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    /// Coalesced camera/vehicle-toggle changes (ball cam, behind-view, driving)
    /// grouped by player — the player id is stored once and each entry holds
    /// that player's frame-ordered changes, rather than a value per frame.
    #[ts(as = "Vec<(crate::interop::ts_bindings::RemoteIdTs, Vec<PlayerCameraStateChange>)>")]
    pub player_camera_events: Vec<(PlayerId, Vec<PlayerCameraStateChange>)>,
    /// Exact player stat counter increments observed during the replay
    pub player_stat_events: Vec<PlayerStatEvent>,
    /// Exact goal events observed during the replay
    pub goal_events: Vec<GoalEvent>,
    /// Replay-authored tick marks/bookmarks from the replay body
    pub replay_tick_marks: Vec<ReplayTickMark>,
}

impl ReplayData {
    /// Serializes the replay data to a JSON string.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing either the JSON string representation
    /// of the replay data or a [`serde_json::Error`] if serialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use subtr_actor::collector::replay_data::ReplayDataCollector;
    /// use boxcars::ParserBuilder;
    ///
    /// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
    /// let replay = ParserBuilder::new(&data).parse().unwrap();
    /// let collector = ReplayDataCollector::new();
    /// let replay_data = collector.get_replay_data(&replay).unwrap();
    ///
    /// let json_string = replay_data.as_json().unwrap();
    /// println!("Replay as JSON: {}", json_string);
    /// ```
    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the replay data to a pretty-printed JSON string.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing either the pretty-printed JSON string
    /// representation of the replay data or a [`serde_json::Error`] if serialization fails.
    pub fn as_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn replay_tick_marks(
    replay: &boxcars::Replay,
    metadata_frames: &[MetadataFrame],
) -> Vec<ReplayTickMark> {
    replay
        .tick_marks
        .iter()
        .map(|tick_mark| ReplayTickMark {
            description: tick_mark.description.clone(),
            frame: tick_mark.frame,
            time: usize::try_from(tick_mark.frame)
                .ok()
                .and_then(|frame| metadata_frames.get(frame))
                .map(|frame| frame.time),
        })
        .collect()
}

/// Groups the processor's flat `(player, change)` camera stream by player,
/// preserving first-appearance player order and per-player frame order, so the
/// serialized form stores each player id once instead of per change.
pub(crate) fn group_player_camera_events(
    events: &[(PlayerId, PlayerCameraStateChange)],
) -> Vec<(PlayerId, Vec<PlayerCameraStateChange>)> {
    let mut grouped: Vec<(PlayerId, Vec<PlayerCameraStateChange>)> = Vec::new();
    for (player_id, change) in events {
        if let Some((_, changes)) = grouped.iter_mut().find(|(id, _)| id == player_id) {
            changes.push(change.clone());
        } else {
            grouped.push((player_id.clone(), vec![change.clone()]));
        }
    }
    grouped
}

#[cfg(test)]
pub(crate) fn player_stat_events_with_shot_saves(
    player_stat_events: &[PlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    player_stat_events_with_shot_saves_and_frame_data(player_stat_events, None, None)
}

fn player_stat_events_with_shot_saves_and_frame_data(
    player_stat_events: &[PlayerStatEvent],
    frame_data: Option<&FrameData>,
    touch_events: Option<&[TouchEvent]>,
) -> Vec<PlayerStatEvent> {
    const MAX_SHOT_SAVE_LINK_SECONDS: f32 = 3.0;

    let mut annotated_events = player_stat_events.to_vec();
    let mut pending_shot_indices: Vec<usize> = Vec::new();

    for index in 0..annotated_events.len() {
        let current_time = annotated_events[index].time;
        pending_shot_indices.retain(|shot_index| {
            current_time - annotated_events[*shot_index].time <= MAX_SHOT_SAVE_LINK_SECONDS
        });

        match annotated_events[index].kind {
            PlayerStatEventKind::Shot => {
                if annotated_events[index].shot.is_some() {
                    pending_shot_indices.push(index);
                }
            }
            PlayerStatEventKind::Save => {
                let save = ShotSaveMetadata {
                    time: annotated_events[index].time,
                    frame: annotated_events[index].frame,
                    player: annotated_events[index].player.clone(),
                    player_position: annotated_events[index].player_position,
                    is_team_0: annotated_events[index].is_team_0,
                };
                let Some(pending_position) = pending_shot_indices.iter().rposition(|shot_index| {
                    let shot_event = &annotated_events[*shot_index];
                    if shot_event.is_team_0 == annotated_events[index].is_team_0 {
                        return false;
                    }
                    let save_time_after_shot = annotated_events[index].time - shot_event.time;
                    if save_time_after_shot <= 0.0
                        || save_time_after_shot > MAX_SHOT_SAVE_LINK_SECONDS
                    {
                        return false;
                    }
                    shot_event
                        .shot
                        .as_ref()
                        .and_then(|shot| shot.projected_goal_line_crossing.as_ref())
                        .is_none_or(|crossing| {
                            shot_goal_line_crossing_is_after_save_reference(
                                shot_event,
                                &save,
                                crossing,
                                touch_events,
                            )
                        })
                }) else {
                    continue;
                };
                let shot_index = pending_shot_indices.remove(pending_position);
                let should_estimate_crossing = annotated_events[shot_index]
                    .shot
                    .as_ref()
                    .is_some_and(|shot| {
                        shot.projected_goal_line_crossing
                            .as_ref()
                            .is_none_or(|crossing| !crossing.inside_goal_mouth)
                    });
                let estimated_crossing = should_estimate_crossing.then(|| {
                    frame_data.and_then(|frame_data| {
                        estimate_saved_shot_goal_line_crossing(
                            &annotated_events[shot_index],
                            &save,
                            frame_data,
                            touch_events,
                        )
                    })
                });
                let unavailable_reason = estimated_crossing
                    .as_ref()
                    .is_none_or(Option::is_none)
                    .then(|| {
                        frame_data.and_then(|frame_data| {
                            saved_shot_goal_line_crossing_unavailable_reason(
                                &annotated_events[shot_index],
                                &save,
                                frame_data,
                                touch_events,
                            )
                        })
                    });
                if let Some(shot) = annotated_events[shot_index].shot.as_mut() {
                    if let Some(Some(estimated_crossing)) = estimated_crossing {
                        shot.projected_goal_target_hit = Some(
                            ShotGoalTargetHit::from_goal_line_crossing(&estimated_crossing),
                        );
                        shot.projected_goal_line_crossing = Some(estimated_crossing);
                        shot.projected_goal_line_crossing_unavailable_reason = None;
                    } else if shot
                        .projected_goal_line_crossing
                        .as_ref()
                        .is_some_and(saved_shot_crossing_is_unphysical_free_flight)
                    {
                        shot.projected_goal_line_crossing = None;
                    }
                    if shot.projected_goal_line_crossing.is_none() {
                        if let Some(Some(unavailable_reason)) = unavailable_reason {
                            shot.projected_goal_line_crossing_unavailable_reason =
                                Some(unavailable_reason);
                        }
                    } else {
                        shot.projected_goal_line_crossing_unavailable_reason = None;
                    }
                    shot.resulting_save = Some(save);
                }
            }
            PlayerStatEventKind::Assist => {}
        }
    }

    annotated_events
}

fn estimate_saved_shot_goal_line_crossing(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    frame_data: &FrameData,
    touch_events: Option<&[TouchEvent]>,
) -> Option<ShotGoalLineCrossing> {
    const MAX_SAVE_TOUCH_STAT_LAG_SECONDS: f32 = 0.25;

    let prediction_window = saved_shot_prediction_window(shot_event, save, touch_events);
    estimate_saved_shot_goal_line_crossing_in_window(shot_event, frame_data, prediction_window)
        .or_else(|| {
            let lagged_prediction_window = saved_shot_prediction_window_with_save_touch_lag(
                shot_event,
                save,
                touch_events,
                MAX_SAVE_TOUCH_STAT_LAG_SECONDS,
            );
            (lagged_prediction_window.has_save_touch
                && !prediction_window.has_save_touch
                && lagged_prediction_window.estimation_time < prediction_window.shot_time)
                .then(|| {
                    estimate_saved_shot_goal_line_crossing_in_window(
                        shot_event,
                        frame_data,
                        lagged_prediction_window,
                    )
                })
                .flatten()
        })
}

fn estimate_saved_shot_goal_line_crossing_in_window(
    shot_event: &PlayerStatEvent,
    frame_data: &FrameData,
    prediction_window: SavedShotPredictionWindow,
) -> Option<ShotGoalLineCrossing> {
    const MAX_PRE_SAVE_LOOKBACK_SECONDS: f32 = 3.0;
    const MAX_NO_TOUCH_SHOT_STAT_LAG_SECONDS: f32 = 0.1;
    const FLOAT_EPSILON: f32 = 0.0001;

    shot_event.shot.as_ref()?;

    let target_direction = if shot_event.is_team_0 { 1.0 } else { -1.0 };
    let estimation_frame = prediction_window
        .estimation_frame
        .min(frame_data.ball_data.frames.len().saturating_sub(1));
    let mut fallback_crossing = None;
    for frame_index in (0..=estimation_frame).rev() {
        let Some(metadata) = frame_data.metadata_frames.get(frame_index) else {
            continue;
        };
        if metadata.time > prediction_window.estimation_time + FLOAT_EPSILON {
            continue;
        }
        if prediction_window.estimation_time - metadata.time > MAX_PRE_SAVE_LOOKBACK_SECONDS {
            break;
        }
        if prediction_window.has_inferred_shot_touch
            && metadata.time + FLOAT_EPSILON < prediction_window.shot_time
        {
            break;
        }

        let Some(BallFrame::Data { rigid_body }) = frame_data.ball_data.frames.get(frame_index)
        else {
            continue;
        };
        let Some(velocity) = rigid_body.linear_velocity else {
            continue;
        };
        if target_direction * velocity.y <= 0.0 {
            continue;
        }

        let Some(mut crossing) = ShotGoalLineCrossing::predict_saved_shot_from_rigid_body(
            shot_event.is_team_0,
            rigid_body,
        ) else {
            continue;
        };
        let crossing_time = metadata.time + crossing.time_after_shot;
        let mut prediction_start_time = prediction_window.shot_time;
        let mut prediction_start_frame = prediction_window.shot_frame;
        if crossing_time <= prediction_window.shot_time + FLOAT_EPSILON {
            if prediction_window.has_inferred_shot_touch
                || prediction_window.has_save_touch
                || prediction_window.shot_time - crossing_time > MAX_NO_TOUCH_SHOT_STAT_LAG_SECONDS
                || crossing_time <= metadata.time + FLOAT_EPSILON
            {
                continue;
            }
            prediction_start_time = metadata.time;
            prediction_start_frame = frame_index;
        }
        if prediction_window.has_save_touch
            && crossing_time <= prediction_window.estimation_time + FLOAT_EPSILON
        {
            continue;
        }
        crossing.time_after_shot = crossing_time - prediction_start_time;
        crossing.prediction_start_time = Some(prediction_start_time);
        crossing.prediction_start_frame = Some(prediction_start_frame);

        if crossing.inside_goal_mouth {
            return Some(crossing);
        }
        fallback_crossing.get_or_insert(crossing);
    }

    fallback_crossing
}

fn saved_shot_goal_line_crossing_unavailable_reason(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    frame_data: &FrameData,
    touch_events: Option<&[TouchEvent]>,
) -> Option<ShotGoalLineCrossingUnavailableReason> {
    let prediction_window = saved_shot_prediction_window(shot_event, save, touch_events);
    Some(saved_shot_goal_line_crossing_unavailable_reason_in_window(
        shot_event,
        save,
        frame_data,
        prediction_window,
    ))
}

fn saved_shot_goal_line_crossing_unavailable_reason_in_window(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    frame_data: &FrameData,
    prediction_window: SavedShotPredictionWindow,
) -> ShotGoalLineCrossingUnavailableReason {
    const MAX_PRE_SAVE_LOOKBACK_SECONDS: f32 = 3.0;
    const FLOAT_EPSILON: f32 = 0.0001;

    let target_direction = if shot_event.is_team_0 { 1.0 } else { -1.0 };
    let estimation_frame = prediction_window
        .estimation_frame
        .min(frame_data.ball_data.frames.len().saturating_sub(1));
    let mut saw_velocity = false;
    let mut inbound_frame_count = 0;
    let mut projected_inbound_frame_count = 0;
    let mut unphysical_free_flight_count = 0;
    let mut crossing_before_or_at_prediction_start_count = 0;
    let mut crossing_before_or_at_save_touch_count = 0;
    let mut crossing_before_or_at_save_count = 0;

    for frame_index in (0..=estimation_frame).rev() {
        let Some(metadata) = frame_data.metadata_frames.get(frame_index) else {
            continue;
        };
        if metadata.time > prediction_window.estimation_time + FLOAT_EPSILON {
            continue;
        }
        if prediction_window.estimation_time - metadata.time > MAX_PRE_SAVE_LOOKBACK_SECONDS {
            break;
        }
        if prediction_window.has_inferred_shot_touch
            && metadata.time + FLOAT_EPSILON < prediction_window.shot_time
        {
            break;
        }

        let Some(BallFrame::Data { rigid_body }) = frame_data.ball_data.frames.get(frame_index)
        else {
            continue;
        };
        let Some(velocity) = rigid_body.linear_velocity else {
            continue;
        };
        saw_velocity = true;
        if target_direction * velocity.y <= 0.0 {
            continue;
        }

        inbound_frame_count += 1;
        let Some((crossing_time, unphysical_free_flight)) =
            saved_shot_diagnostic_crossing_time(shot_event.is_team_0, rigid_body)
        else {
            continue;
        };
        projected_inbound_frame_count += 1;
        if unphysical_free_flight {
            unphysical_free_flight_count += 1;
            continue;
        }

        let absolute_crossing_time = metadata.time + crossing_time;
        if absolute_crossing_time <= prediction_window.shot_time + FLOAT_EPSILON {
            crossing_before_or_at_prediction_start_count += 1;
            continue;
        }
        if prediction_window.has_save_touch
            && absolute_crossing_time <= prediction_window.estimation_time + FLOAT_EPSILON
        {
            crossing_before_or_at_save_touch_count += 1;
            continue;
        }
        if absolute_crossing_time <= save.time + FLOAT_EPSILON {
            crossing_before_or_at_save_count += 1;
            continue;
        }

        return ShotGoalLineCrossingUnavailableReason::NoUsableProjection;
    }

    if !saw_velocity {
        return ShotGoalLineCrossingUnavailableReason::NoBallVelocity;
    }
    if inbound_frame_count == 0 {
        return ShotGoalLineCrossingUnavailableReason::NoGoalwardBallBeforeSaveReference;
    }
    if projected_inbound_frame_count == 0 {
        return ShotGoalLineCrossingUnavailableReason::NoGoalLineCrossingBeforeSaveReference;
    }
    if unphysical_free_flight_count == projected_inbound_frame_count {
        return ShotGoalLineCrossingUnavailableReason::OnlyUnphysicalFreeFlightCrossings;
    }
    if crossing_before_or_at_prediction_start_count == projected_inbound_frame_count {
        return ShotGoalLineCrossingUnavailableReason::CrossingsBeforePredictionStart;
    }
    if crossing_before_or_at_save_touch_count == projected_inbound_frame_count {
        return ShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveTouch;
    }
    if crossing_before_or_at_save_count == projected_inbound_frame_count {
        return ShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveStat;
    }

    ShotGoalLineCrossingUnavailableReason::NoUsableProjection
}

fn saved_shot_diagnostic_crossing_time(
    is_team_0: bool,
    rigid_body: &boxcars::RigidBody,
) -> Option<(f32, bool)> {
    let crossing_config = BallGoalLineCrossingConfig::attacking_goal(is_team_0);
    let surfaces = standard_soccar_goal_line_prediction_field_surfaces();
    predict_ball_with_surface_bounces_goal_line_crossing(
        rigid_body,
        crossing_config,
        BallTrajectoryConfig::STANDARD_SOCCAR,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    )
    .map(|crossing| (crossing.time, false))
    .or_else(|| {
        predict_free_flight_goal_line_crossing(
            rigid_body,
            crossing_config,
            BallTrajectoryConfig::STANDARD_SOCCAR,
        )
        .map(|crossing| {
            (
                crossing.time,
                crossing.position.z < STANDARD_BALL_RADIUS - STANDARD_GOAL_MOUTH_TRAJECTORY_MARGIN,
            )
        })
    })
}

fn saved_shot_crossing_is_unphysical_free_flight(crossing: &ShotGoalLineCrossing) -> bool {
    matches!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::FreeFlight
            | ShotGoalLineCrossingPredictionKind::SavedShotPreSaveFreeFlight
    ) && crossing.position.z < STANDARD_BALL_RADIUS - STANDARD_GOAL_MOUTH_TRAJECTORY_MARGIN
}

fn shot_goal_line_crossing_is_after_save_reference(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    crossing: &ShotGoalLineCrossing,
    touch_events: Option<&[TouchEvent]>,
) -> bool {
    const FLOAT_EPSILON: f32 = 0.0001;

    let crossing_time =
        crossing.prediction_start_time.unwrap_or(shot_event.time) + crossing.time_after_shot;
    let save_reference_time =
        saved_shot_prediction_window(shot_event, save, touch_events).save_reference_time();
    crossing_time > save_reference_time + FLOAT_EPSILON
}

#[derive(Debug, Clone, Copy)]
struct SavedShotPredictionWindow {
    shot_frame: usize,
    shot_time: f32,
    has_inferred_shot_touch: bool,
    has_save_touch: bool,
    estimation_frame: usize,
    estimation_time: f32,
}

impl SavedShotPredictionWindow {
    fn save_reference_time(self) -> f32 {
        if self.has_save_touch {
            self.estimation_time
        } else {
            self.estimation_time.max(self.shot_time)
        }
    }
}

fn saved_shot_prediction_window(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    touch_events: Option<&[TouchEvent]>,
) -> SavedShotPredictionWindow {
    saved_shot_prediction_window_with_save_touch_lag(shot_event, save, touch_events, 0.0)
}

fn saved_shot_prediction_window_with_save_touch_lag(
    shot_event: &PlayerStatEvent,
    save: &ShotSaveMetadata,
    touch_events: Option<&[TouchEvent]>,
    max_save_touch_stat_lag_seconds: f32,
) -> SavedShotPredictionWindow {
    const FLOAT_EPSILON: f32 = 0.0001;
    const MAX_SHOT_TOUCH_LOOKBACK_SECONDS: f32 = 3.0;

    let save_touch = touch_events.and_then(|touch_events| {
        let player_touch = touch_events.iter().rev().find(|touch| {
            touch.team_is_team_0 == save.is_team_0
                && touch.player.as_ref() == Some(&save.player)
                && touch.time >= shot_event.time - max_save_touch_stat_lag_seconds - FLOAT_EPSILON
                && touch.time <= save.time + FLOAT_EPSILON
        });
        let team_touch = || {
            touch_events.iter().rev().find(|touch| {
                touch.team_is_team_0 == save.is_team_0
                    && touch.time
                        >= shot_event.time - max_save_touch_stat_lag_seconds - FLOAT_EPSILON
                    && touch.time <= save.time + FLOAT_EPSILON
            })
        };
        player_touch.or_else(team_touch)
    });
    let shot_touch = touch_events.and_then(|touch_events| {
        let player_touch = touch_events.iter().rev().find(|touch| {
            touch.team_is_team_0 == shot_event.is_team_0
                && touch.player.as_ref() == Some(&shot_event.player)
                && touch.time >= shot_event.time - MAX_SHOT_TOUCH_LOOKBACK_SECONDS - FLOAT_EPSILON
                && touch.time <= shot_event.time + FLOAT_EPSILON
        });
        let team_touch = || {
            touch_events.iter().rev().find(|touch| {
                touch.team_is_team_0 == shot_event.is_team_0
                    && touch.time
                        >= shot_event.time - MAX_SHOT_TOUCH_LOOKBACK_SECONDS - FLOAT_EPSILON
                    && touch.time <= shot_event.time + FLOAT_EPSILON
            })
        };
        player_touch.or_else(team_touch)
    });

    let (estimation_frame, estimation_time) = save_touch
        .map(|touch| {
            let frame = if touch.frame > 0 {
                touch.frame - 1
            } else {
                touch.frame
            };
            (frame, touch.time)
        })
        .unwrap_or((save.frame, save.time));
    let has_save_touch = save_touch.is_some();
    let inferred_shot_touch =
        shot_touch.filter(|touch| touch.time <= estimation_time + FLOAT_EPSILON);
    let has_inferred_shot_touch = inferred_shot_touch.is_some();
    let (shot_frame, shot_time) = inferred_shot_touch
        .map(|touch| (touch.frame, touch.time))
        .unwrap_or((shot_event.frame, shot_event.time));

    if shot_frame <= estimation_frame {
        SavedShotPredictionWindow {
            shot_frame,
            shot_time,
            has_inferred_shot_touch,
            has_save_touch,
            estimation_frame,
            estimation_time,
        }
    } else {
        SavedShotPredictionWindow {
            shot_frame: shot_event.frame,
            shot_time: shot_event.time,
            has_inferred_shot_touch: false,
            has_save_touch,
            estimation_frame,
            estimation_time,
        }
    }
}

impl FrameData {
    /// Creates a new empty [`FrameData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`FrameData`] with empty ball data, player data, and metadata frames.
    fn new() -> Self {
        FrameData {
            ball_data: BallData::new(),
            players: Vec::new(),
            metadata_frames: Vec::new(),
        }
    }

    /// Returns the total number of frames in this frame data.
    ///
    /// # Returns
    ///
    /// Returns the number of metadata frames, which represents the total frame count.
    pub fn frame_count(&self) -> usize {
        self.metadata_frames.len()
    }

    /// Returns the duration of the replay in seconds.
    ///
    /// # Returns
    ///
    /// Returns the time of the last frame, or 0.0 if no frames exist.
    pub fn duration(&self) -> f32 {
        self.metadata_frames.last().map(|f| f.time).unwrap_or(0.0)
    }

    /// Adds a complete frame of data to the frame data structure.
    ///
    /// This method adds metadata, ball data, and player data for a single frame
    /// to their respective collections, maintaining frame synchronization across
    /// all data types.
    ///
    /// # Arguments
    ///
    /// * `frame_metadata` - The metadata for this frame (time, game state, etc.)
    /// * `ball_frame` - The ball state for this frame
    /// * `player_frames` - Player state data for all players in this frame
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// May return a [`SubtrActorError`] if frame data cannot be processed correctly.
    fn add_frame(
        &mut self,
        frame_metadata: MetadataFrame,
        ball_frame: BallFrame,
        player_frames: Vec<(PlayerId, PlayerFrame)>,
    ) -> SubtrActorResult<()> {
        let frame_index = self.metadata_frames.len();
        self.metadata_frames.push(frame_metadata);
        self.ball_data.add_frame(frame_index, ball_frame);
        for (player_id, frame) in player_frames {
            self.players
                .get_entry(player_id)
                .or_insert_with(PlayerData::new)
                .add_frame(frame_index, frame)
        }
        Ok(())
    }
}

/// A collector that extracts comprehensive frame-by-frame data from Rocket League replays.
///
/// [`ReplayDataCollector`] implements the [`Collector`] trait to process replay frames
/// and extract detailed information about ball movement, player actions, and game state.
/// It builds a complete [`ReplayData`] structure containing all available information
/// from the replay.
///
/// # Usage
///
/// The collector is designed to be used with the [`ReplayProcessor`] to extract
/// comprehensive replay data:
///
/// ```rust
/// use subtr_actor::collector::replay_data::ReplayDataCollector;
/// use boxcars::ParserBuilder;
///
/// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
/// let replay = ParserBuilder::new(&data).parse().unwrap();
///
/// let collector = ReplayDataCollector::new();
/// let replay_data = collector.get_replay_data(&replay).unwrap();
///
/// // Process the extracted data
/// for (frame_idx, metadata) in replay_data.frame_data.metadata_frames.iter().enumerate() {
///     println!("Frame {}: Time={:.2}s, Remaining={}s",
///              frame_idx, metadata.time, metadata.seconds_remaining);
/// }
/// ```
///
/// # Fields
///
/// * `frame_data` - Internal storage for frame-by-frame data during collection
pub struct ReplayDataCollector {
    /// Internal storage for frame-by-frame data during collection
    frame_data: FrameData,
}

impl Default for ReplayDataCollector {
    /// Creates a default [`ReplayDataCollector`] instance.
    ///
    /// This is equivalent to calling [`ReplayDataCollector::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayDataCollector {
    /// Creates a new [`ReplayDataCollector`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new collector ready to process replay frames.
    pub fn new() -> Self {
        ReplayDataCollector {
            frame_data: FrameData::new(),
        }
    }

    /// Consumes the collector and returns the collected frame data.
    ///
    /// # Returns
    ///
    /// Returns the [`FrameData`] containing all processed frame information.
    pub fn get_frame_data(self) -> FrameData {
        self.frame_data
    }

    pub fn into_replay_data(self, processor: ReplayProcessor<'_>) -> SubtrActorResult<ReplayData> {
        let meta = processor.get_replay_meta()?;
        let frame_data = self.get_frame_data();
        Ok(ReplayData {
            meta,
            demolish_infos: processor.demolishes().to_vec(),
            boost_pad_events: processor.boost_pad_events().to_vec(),
            boost_pads: processor.resolved_boost_pads(),
            touch_events: processor.touch_events().to_vec(),
            dodge_refreshed_events: processor.dodge_refreshed_events().to_vec(),
            player_camera_events: group_player_camera_events(processor.player_camera_events()),
            player_stat_events: player_stat_events_with_shot_saves_and_frame_data(
                processor.player_stat_events(),
                Some(&frame_data),
                Some(processor.touch_events()),
            ),
            goal_events: processor.goal_events().to_vec(),
            replay_tick_marks: replay_tick_marks(processor.replay, &frame_data.metadata_frames),
            frame_data,
        })
    }

    /// Processes a replay and returns complete replay data.
    ///
    /// This method processes the entire replay using a [`ReplayProcessor`] and
    /// extracts all available information including frame-by-frame data, metadata,
    /// and special events like demolitions.
    ///
    /// # Arguments
    ///
    /// * `replay` - The parsed replay data from the [`boxcars`] library
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing the complete [`ReplayData`] structure
    /// with all extracted information.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - The replay processor cannot be created
    /// - Frame processing fails
    /// - Replay metadata cannot be extracted
    ///
    /// # Example
    ///
    /// ```rust
    /// use subtr_actor::collector::replay_data::ReplayDataCollector;
    /// use boxcars::ParserBuilder;
    ///
    /// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
    /// let replay = ParserBuilder::new(&data).parse().unwrap();
    ///
    /// let collector = ReplayDataCollector::new();
    /// let replay_data = collector.get_replay_data(&replay).unwrap();
    ///
    /// println!("Processed {} frames", replay_data.frame_data.frame_count());
    /// ```
    pub fn get_replay_data(mut self, replay: &boxcars::Replay) -> SubtrActorResult<ReplayData> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process_all(&mut [&mut self])?;
        self.into_replay_data(processor)
    }

    /// Extracts player frame data for all players at the specified time.
    ///
    /// This method iterates through all players in the replay and extracts their
    /// state information at the given time, returning a vector of player frames
    /// indexed by player ID.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `current_time` - The time in seconds at which to extract player states
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing a vector of tuples with player IDs
    /// and their corresponding [`PlayerFrame`] data.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if player frame data cannot be extracted.
    fn get_player_frames(
        &self,
        processor: &dyn ProcessorView,
        current_time: f32,
    ) -> SubtrActorResult<Vec<(PlayerId, PlayerFrame)>> {
        Ok(processor
            .iter_player_ids_in_order()
            .map(|player_id| {
                (
                    player_id.clone(),
                    PlayerFrame::new_from_processor(processor, player_id, current_time)
                        .unwrap_or(PlayerFrame::Empty),
                )
            })
            .collect())
    }
}

impl Collector for ReplayDataCollector {
    /// Processes a single frame of the replay and extracts all relevant data.
    ///
    /// This method is called by the [`ReplayProcessor`] for each frame in the replay.
    /// It extracts metadata, ball state, and player state information and adds them
    /// to the internal frame data structure.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data and context
    /// * `_frame` - The current frame data (unused in this implementation)
    /// * `_frame_number` - The current frame number (unused in this implementation)
    /// * `current_time` - The current time in seconds since the start of the replay
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing [`TimeAdvance::NextFrame`] to
    /// indicate that processing should continue to the next frame.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - Metadata frame cannot be created
    /// - Player frame data cannot be extracted
    /// - Frame data cannot be added to the collection
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let metadata_frame = MetadataFrame::new_from_processor(processor, current_time)?;
        let ball_frame = BallFrame::new_from_processor(processor, current_time);
        let player_frames = self.get_player_frames(processor, current_time)?;
        self.frame_data
            .add_frame(metadata_frame, ball_frame, player_frames)?;
        Ok(TimeAdvance::NextFrame)
    }
}
