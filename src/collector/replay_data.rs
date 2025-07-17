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
//! let data = std::fs::read("assets/replays/new_boost_format.replay").unwrap();
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
/// The ball can either be in an empty state (when sleeping or when ball syncing
/// is disabled) or contain full physics data including position, rotation, and
/// velocity information.
///
/// # Variants
///
/// - [`Empty`](BallFrame::Empty) - Indicates the ball is sleeping or ball syncing is disabled
/// - [`Data`](BallFrame::Data) - Contains the ball's rigid body physics information
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BallFrame {
    /// Empty frame indicating the ball is sleeping or ball syncing is disabled
    Empty,
    /// Frame containing the ball's rigid body physics data
    Data {
        /// The ball's rigid body containing position, rotation, and velocity information
        rigid_body: boxcars::RigidBody,
    },
}

impl BallFrame {
    /// Creates a new [`BallFrame`] from a [`ReplayProcessor`] at the specified time.
    ///
    /// This method extracts the ball's state from the replay processor, handling
    /// cases where ball syncing is disabled or the ball is in a sleeping state.
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
    /// - The ball is in a sleeping state
    ///
    /// Otherwise returns [`Data`](BallFrame::Data) containing the ball's rigid body.
    fn new_from_processor(processor: &ReplayProcessor, current_time: f32) -> Self {
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
    /// Returns [`Empty`](BallFrame::Empty) if the rigid body is in a sleeping state,
    /// otherwise returns [`Data`](BallFrame::Data) containing the rigid body.
    fn new_from_rigid_body(rigid_body: boxcars::RigidBody) -> Self {
        if rigid_body.sleeping {
            Self::Empty
        } else {
            Self::Data { rigid_body }
        }
    }
}

/// Represents a player's state for a single frame in a Rocket League replay.
///
/// Contains comprehensive information about a player's position, movement,
/// and control inputs during a specific frame of the replay.
///
/// # Variants
///
/// - [`Empty`](PlayerFrame::Empty) - Indicates the player is inactive or sleeping
/// - [`Data`](PlayerFrame::Data) - Contains the player's complete state information
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PlayerFrame {
    /// Empty frame indicating the player is inactive or sleeping
    Empty,
    /// Frame containing the player's complete state data
    Data {
        /// The player's rigid body containing position, rotation, and velocity information
        rigid_body: boxcars::RigidBody,
        /// The player's current boost amount (0.0 to 1.0)
        boost_amount: f32,
        /// Whether the player is actively using boost
        boost_active: bool,
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
    /// Returns a [`SubtrActorResult`] containing a [`PlayerFrame`] which will be:
    /// - [`Empty`](PlayerFrame::Empty) if the player's rigid body is in a sleeping state
    /// - [`Data`](PlayerFrame::Data) containing the player's complete state information
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - The player's rigid body cannot be retrieved
    /// - The player's boost level cannot be accessed
    /// - Other player state information is inaccessible
    fn new_from_processor(
        processor: &ReplayProcessor,
        player_id: &PlayerId,
        current_time: f32,
    ) -> SubtrActorResult<Self> {
        let rigid_body =
            processor.get_interpolated_player_rigid_body(player_id, current_time, 0.0)?;

        if rigid_body.sleeping {
            return Ok(PlayerFrame::Empty);
        }

        let boost_amount = processor.get_player_boost_level(player_id)?;
        let boost_active = processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1;
        let jump_active = processor.get_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let double_jump_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let dodge_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1;

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
            jump_active,
            double_jump_active,
            dodge_active,
            player_name,
            team,
            is_team_0,
        ))
    }

    /// Creates a [`PlayerFrame`] from the provided data components.
    ///
    /// # Arguments
    ///
    /// * `rigid_body` - The player's rigid body physics information
    /// * `boost_amount` - The player's current boost level (0.0 to 1.0)
    /// * `boost_active` - Whether the player is actively using boost
    /// * `jump_active` - Whether the player is actively jumping
    /// * `double_jump_active` - Whether the player is performing a double jump
    /// * `dodge_active` - Whether the player is performing a dodge maneuver
    /// * `player_name` - The player's name, if available
    /// * `team` - The player's team number, if available
    /// * `is_team_0` - Whether the player is on team 0, if available
    ///
    /// # Returns
    ///
    /// Returns [`Empty`](PlayerFrame::Empty) if the rigid body is sleeping,
    /// otherwise returns [`Data`](PlayerFrame::Data) with all provided information.
    #[allow(clippy::too_many_arguments)]
    fn from_data(
        rigid_body: boxcars::RigidBody,
        boost_amount: f32,
        boost_active: bool,
        jump_active: bool,
        double_jump_active: bool,
        dodge_active: bool,
        player_name: Option<String>,
        team: Option<i32>,
        is_team_0: Option<bool>,
    ) -> Self {
        if rigid_body.sleeping {
            Self::Empty
        } else {
            Self::Data {
                rigid_body,
                boost_amount,
                boost_active,
                jump_active,
                double_jump_active,
                dodge_active,
                player_name,
                team,
                is_team_0,
            }
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetadataFrame {
    /// The current time in seconds since the start of the replay
    pub time: f32,
    /// The number of seconds remaining in the current game period
    pub seconds_remaining: i32,
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
    /// Returns a [`SubtrActorError`] if the seconds remaining cannot be retrieved
    /// from the processor.
    fn new_from_processor(processor: &ReplayProcessor, time: f32) -> SubtrActorResult<Self> {
        Ok(Self::new(time, processor.get_seconds_remaining()?))
    }

    /// Creates a new [`MetadataFrame`] with the specified time and seconds remaining.
    ///
    /// # Arguments
    ///
    /// * `time` - The current time in seconds since the start of the replay
    /// * `seconds_remaining` - The number of seconds remaining in the current game period
    ///
    /// # Returns
    ///
    /// Returns a new [`MetadataFrame`] with the provided values.
    fn new(time: f32, seconds_remaining: i32) -> Self {
        MetadataFrame {
            time,
            seconds_remaining,
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameData {
    /// All ball state information across all frames
    pub ball_data: BallData,
    /// Player data for each player, indexed by PlayerId
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
///
/// # Example
///
/// ```rust
/// use subtr_actor::collector::replay_data::ReplayDataCollector;
/// use boxcars::ParserBuilder;
///
/// let data = std::fs::read("assets/replays/new_boost_format.replay").unwrap();
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayData {
    /// All frame-by-frame data including ball, player, and metadata information
    pub frame_data: FrameData,
    /// Replay metadata including player information, game settings, and statistics
    pub meta: ReplayMeta,
    /// Information about all demolition events that occurred during the replay
    pub demolish_infos: Vec<DemolishInfo>,
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
    /// let data = std::fs::read("assets/replays/new_boost_format.replay").unwrap();
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
/// let data = std::fs::read("assets/replays/new_boost_format.replay").unwrap();
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
    /// let data = std::fs::read("assets/replays/new_boost_format.replay").unwrap();
    /// let replay = ParserBuilder::new(&data).parse().unwrap();
    ///
    /// let collector = ReplayDataCollector::new();
    /// let replay_data = collector.get_replay_data(&replay).unwrap();
    ///
    /// println!("Processed {} frames", replay_data.frame_data.frame_count());
    /// ```
    pub fn get_replay_data(mut self, replay: &boxcars::Replay) -> SubtrActorResult<ReplayData> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        let meta = processor.get_replay_meta()?;
        Ok(ReplayData {
            meta,
            demolish_infos: processor.demolishes,
            frame_data: self.get_frame_data(),
        })
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
        processor: &ReplayProcessor,
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
        processor: &ReplayProcessor,
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
