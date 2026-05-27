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

#[path = "replay_data_ball_data.rs"]
mod ball_data;
#[path = "replay_data_ball_frame.rs"]
mod ball_frame;
#[path = "replay_data_collector.rs"]
mod collector;
#[path = "replay_data_collector_output.rs"]
mod collector_output;
#[path = "replay_data_collector_process.rs"]
mod collector_process;
#[path = "replay_data_frame_data.rs"]
mod frame_data;
#[path = "replay_data_metadata_frame.rs"]
mod metadata_frame;
#[path = "replay_data_payload.rs"]
mod payload;
#[path = "replay_data_player_data.rs"]
mod player_data;
#[path = "replay_data_player_frame.rs"]
mod player_frame;
#[path = "replay_data_player_frame_input.rs"]
mod player_frame_input;

pub use ball_data::BallData;
pub use ball_frame::BallFrame;
pub use collector::ReplayDataCollector;
pub use frame_data::FrameData;
pub use metadata_frame::MetadataFrame;
pub use payload::ReplayData;
pub use player_data::PlayerData;
pub use player_frame::PlayerFrame;
