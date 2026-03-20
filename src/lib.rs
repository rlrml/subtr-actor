#![allow(clippy::result_large_err)]

//! # subtr-actor
//!
//! `subtr-actor` turns raw [`boxcars`] replay data into higher-level game
//! state, derived replay events, structured frame payloads, and dense numeric
//! features for analytics and ML workflows.
//!
//! The Rust crate is the source of truth for the replay-processing pipeline.
//! The Python and JavaScript bindings build on the same collector APIs and
//! string-addressable feature registry exposed here.
//!
//! ## Processing model
//!
//! - [`ReplayProcessor`] walks the replay's network frames, models actor state,
//!   and tracks derived replay events such as touches, boost pad pickups,
//!   dodge refreshes, goals, player stat events, and demolishes.
//! - [`Collector`] is the core extension point. Collectors observe the replay
//!   frame by frame and can either process every frame or control sampling via
//!   [`TimeAdvance`].
//! - [`ReplayProcessor::process_all`] lets multiple collectors share a single
//!   replay pass when you want to build several outputs at once.
//! - [`FrameRateDecorator`] and [`CallbackCollector`] provide lightweight
//!   utilities for downsampling a collector or attaching side-effectful hooks
//!   such as progress reporting and debugging.
//!
//! ## Primary output layers
//!
//! - [`ReplayDataCollector`] builds a serde-friendly replay payload with frame
//!   data, replay metadata, and derived event streams suitable for JSON export
//!   and playback UIs.
//! - [`NDArrayCollector`] emits a dense [`::ndarray::Array2`] with replay
//!   metadata and headers. It supports both explicit feature adders and the
//!   string-based registry exposed through [`NDArrayCollector::from_strings`]
//!   and [`NDArrayCollector::from_strings_typed`].
//! - [`StatsTimelineCollector`] accumulates reducer-based replay statistics
//!   frame by frame and can return either typed snapshots
//!   ([`ReplayStatsTimeline`]) or a dynamic stat-field representation
//!   ([`DynamicReplayStatsTimeline`]).
//!
//! ## Stats and exports
//!
//! The [`stats`] module houses reducer implementations, stat mechanics helpers,
//! and the exported stat-field model built around [`ExportedStat`]. The same
//! export layer is re-exported from [`crate::stats_export`] for compatibility
//! with older import paths.
//!
//! ## Examples
//!
//! ### Collect structured replay data
//!
//! ```no_run
//! use boxcars::ParserBuilder;
//! use subtr_actor::ReplayDataCollector;
//!
//! let bytes = std::fs::read("replay.replay").unwrap();
//! let replay = ParserBuilder::new(&bytes)
//!     .must_parse_network_data()
//!     .on_error_check_crc()
//!     .parse()
//!     .unwrap();
//!
//! let replay_data = ReplayDataCollector::new().get_replay_data(&replay).unwrap();
//! println!("frames: {}", replay_data.frame_data.frame_count());
//! println!("touches: {}", replay_data.touch_events.len());
//! ```
//!
//! ### Build a sampled feature matrix
//!
//! ```no_run
//! use boxcars::ParserBuilder;
//! use subtr_actor::{Collector, FrameRateDecorator, NDArrayCollector};
//!
//! let bytes = std::fs::read("replay.replay").unwrap();
//! let replay = ParserBuilder::new(&bytes)
//!     .must_parse_network_data()
//!     .on_error_check_crc()
//!     .parse()
//!     .unwrap();
//!
//! let mut collector = NDArrayCollector::<f32>::from_strings(
//!     &["BallRigidBody", "CurrentTime"],
//!     &["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
//! )
//! .unwrap();
//!
//! FrameRateDecorator::new_from_fps(30.0, &mut collector)
//!     .process_replay(&replay)
//!     .unwrap();
//!
//! let (meta, features) = collector.get_meta_and_ndarray().unwrap();
//! println!("players: {}", meta.replay_meta.player_count());
//! println!("shape: {:?}", features.raw_dim());
//! ```
//!
//! ### Export dynamic stats timeline snapshots
//!
//! ```no_run
//! use boxcars::ParserBuilder;
//! use subtr_actor::StatsTimelineCollector;
//!
//! let bytes = std::fs::read("replay.replay").unwrap();
//! let replay = ParserBuilder::new(&bytes)
//!     .must_parse_network_data()
//!     .on_error_check_crc()
//!     .parse()
//!     .unwrap();
//!
//! let timeline = StatsTimelineCollector::new()
//!     .get_dynamic_replay_data(&replay)
//!     .unwrap();
//!
//! println!("timeline frames: {}", timeline.frames.len());
//! println!("rush events: {}", timeline.rush_events.len());
//! ```

pub mod actor_state;
pub mod ballchasing;
pub mod collector;
pub mod constants;
pub mod error;
pub mod mechanics;
pub mod processor;
pub mod stats;
pub mod stats_export;
pub mod util;

#[cfg(test)]
mod util_test;

pub use crate::actor_state::*;
pub use crate::collector::*;
pub use crate::constants::*;
pub use crate::error::*;
pub use crate::mechanics::*;
pub use crate::processor::*;
pub use crate::stats::*;
pub use crate::util::*;
