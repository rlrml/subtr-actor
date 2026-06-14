#![allow(clippy::result_large_err)]

//! `subtr-actor` turns raw [`boxcars`](https://docs.rs/boxcars) replay data into
//! higher-level game state, derived replay events, structured frame payloads, and
//! dense numeric features for analytics and ML workflows.
//!
//! - **Higher-level game state** modeled from the raw actor graph
//! - **Frame-by-frame structured data** ready for JSON export and playback UIs
//! - **Dense numeric feature matrices** for ML, built from a string-addressable
//!   feature registry
//! - **Derived events and cumulative stats** — touches, boost pickups, dodge
//!   refreshes, goals, demolishes, and more
//! - **One pipeline, three languages** — the same Rust core drives the Python and
//!   JavaScript/WASM bindings
//!
//! ## Processing model
//!
//! - `ReplayProcessor` walks the replay's network frames, models actor state,
//!   and tracks derived replay events such as touches, boost pad pickups,
//!   dodge refreshes, goals, player stat events, and demolishes.
//! - `Collector` is the core extension point. Collectors observe the replay
//!   frame by frame and can either process every frame or control sampling via
//!   `TimeAdvance`.
//! - `ReplayProcessor::process_all` lets multiple collectors share a single
//!   replay pass when you want to build several outputs at once.
//! - `FrameRateDecorator` and `CallbackCollector` provide lightweight
//!   utilities for downsampling a collector or attaching side-effectful hooks
//!   such as progress reporting and debugging.
//!
//! ## Primary output layers
//!
//! - `ReplayDataCollector` builds a serde-friendly replay payload with frame
//!   data, replay metadata, and derived event streams suitable for JSON export
//!   and playback UIs.
//! - `NDArrayCollector` emits a dense `ndarray::Array2` with replay
//!   metadata and headers. It supports both explicit feature adders and the
//!   string-based registry exposed through `NDArrayCollector::from_strings`
//!   and `NDArrayCollector::from_strings_typed`.
//! - `StatsCollector` accumulates graph-backed replay statistics as a
//!   module-keyed dynamic payload suitable for builtin module selection and
//!   JSON export.
//! - `StatsTimelineEventCollector` accumulates graph-backed replay statistics
//!   as event streams plus lightweight frame scaffolding. This is the preferred
//!   timeline export when callers do not need to serialize full per-frame
//!   partial sums.
//! - `StatsTimelineCollector` preserves the legacy full snapshot timeline
//!   form (`ReplayStatsTimeline`) for parity checks and compatibility.
//!
//! ## Stats and exports
//!
//! The `stats` module houses analysis calculators, graph nodes, stat
//! event calculators, and the exported stat-field model built around
//! `ExportedStat`.
//!
//! ## Architecture / module map
//!
//! Read top-down — each module's own documentation expands on the summary
//! here and links to the collections of implementations it contains.
//!
//! - [`processor`] — the replay-walking core. [`ReplayProcessor`] models actor
//!   state from `boxcars` network frames and tracks derived events, applying a
//!   sequence of per-frame state updaters.
//! - [`collector`] — the output layer. The [`Collector`] trait is the extension
//!   point; built-in collectors are [`ReplayDataCollector`] (structured frames),
//!   [`NDArrayCollector`] ([numeric features][collector::ndarray]), and the
//!   stats-timeline collectors ([`collector::stats`]).
//! - [`stats`] — the analysis layer. A dependency graph of
//!   [analysis nodes][stats::analysis_graph] wraps
//!   [gameplay-event calculators][StatsEvent] that detect mechanics; results
//!   land in accumulators and the [exported stat-field model][stats::export].
//! - [`replay_model`] / [`replay_meta`] — the serde-friendly higher-level game
//!   state and replay metadata produced for export and playback UIs.
//! - [`interop`] — bindings-facing helpers shared by the Python and
//!   JavaScript/WASM wrappers (e.g. the replay-player manifest).
//! - [`util`] — geometry, search, and small data-structure helpers used
//!   throughout the crate.
//!
//! ## Where to find collections of implementations
//!
//! Several parts of the crate are large families of similar types. Each has a
//! catalog in its module documentation, and the shared trait's *Implementors*
//! list is a second way to browse them:
//!
//! | Collection | Module | Shared trait / registry |
//! |---|---|---|
//! | Gameplay-event calculators | [`stats::analysis_graph`] | [`StatsEvent`] |
//! | Analysis-graph nodes | [`stats::analysis_graph`] | [`AnalysisNode`](stats::analysis_graph::AnalysisNode) |
//! | Stat accumulators | [`stats::accumulators`] | (plain accumulation structs) |
//! | Exported stat fields | [`stats::export`] | [`StatFieldProvider`] |
//! | NDArray feature adders | [`collector::ndarray`] | [`FeatureAdder`] family + string registry |
//! | Processor state updaters | [`processor`] | (`impl ReplayProcessor` methods) |
//!
//! ## In-depth guides
//!
//! Longer prose guides are rendered into the API docs under [`guides`]:
//!
//! - [`guides::calculators_and_analysis_nodes`] — the stats runtime DAG layout.
//! - [`guides::stat_confidence`] — how to read exported-stat confidence levels.
//! - [`guides::replay_format_evolution`] — replay-format changes that matter
//!   to parsing.
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
//! ### Export compact event-backed stats timeline
//!
//! ```no_run
//! use boxcars::ParserBuilder;
//! use subtr_actor::StatsTimelineEventCollector;
//!
//! let bytes = std::fs::read("replay.replay").unwrap();
//! let replay = ParserBuilder::new(&bytes)
//!     .must_parse_network_data()
//!     .on_error_check_crc()
//!     .parse()
//!     .unwrap();
//!
//! let timeline = StatsTimelineEventCollector::new()
//!     .get_replay_stats_timeline_scaffold(&replay)
//!     .unwrap();
//!
//! println!("timeline frames: {}", timeline.frames.len());
//! let rush_events = timeline
//!     .events
//!     .events
//!     .iter()
//!     .filter(|event| event.meta.stream == "rush")
//!     .count();
//! println!("rush events: {rush_events}");
//! ```

#[path = "domain/boost_units.rs"]
pub mod boost_units;
pub mod clip;
pub mod collector;
#[path = "domain/error.rs"]
pub mod error;
pub mod interop;
pub mod processor;
#[path = "domain/replay_meta.rs"]
pub mod replay_meta;
#[path = "domain/replay_model.rs"]
pub mod replay_model;
pub mod stats;
pub mod util;

pub mod geometry {
    //! Compatibility re-export for geometry helpers.
    pub use crate::util::geometry::*;
}

pub mod search {
    //! Compatibility re-export for search helpers.
    pub use crate::util::search::*;
}

pub mod actor_state {
    //! Compatibility re-export for processor actor-state types.
    pub use crate::processor::actor_state::*;
}

/// In-depth prose guides, rendered from the repository's `docs/` directory.
///
/// These pages give background and design context that does not belong on any
/// single type. They are documentation-only modules (no code).
pub mod guides {
    /// Map of the stats runtime: how calculators, analysis-graph nodes, and
    /// accumulators fit together into the analysis DAG.
    #[doc = include_str!("../docs/calculators-and-analysis-nodes.md")]
    pub mod calculators_and_analysis_nodes {}

    /// How to interpret the confidence levels attached to exported stats.
    #[doc = include_str!("../docs/stat-confidence.md")]
    pub mod stat_confidence {}

    /// Working notes on Rocket League replay-format changes that affect parsing.
    #[doc = include_str!("../docs/replay-format-evolution.md")]
    pub mod replay_format_evolution {
        // The guide uses markdown link-reference definitions, which rustdoc's
        // bare-URL lint flags even though they render correctly.
        #![allow(rustdoc::bare_urls)]
    }
}

pub use crate::actor_state::*;
pub use crate::boost_units::*;
pub use crate::clip::*;
pub use crate::collector::*;
pub use crate::error::*;
pub use crate::geometry::*;
pub use crate::processor::*;
pub use crate::replay_meta::*;
pub use crate::replay_model::*;
pub use crate::search::*;
pub use crate::stats::*;
pub(crate) use crate::util::vec_map::*;
