//! Stats collector: run the [analysis graph](crate::stats::analysis_graph) over
//! a replay and surface accumulated stats as a module-keyed payload.
//!
//! [`StatsCollector`] selects builtin stats modules (by name), drives the graph,
//! and produces [`CollectedStats`] — suitable for builtin module selection and
//! JSON export. Its output frame shape is pluggable via the [`FrameTransform`]
//! family ([`IdentityFrameTransform`], [`ModuleFrameTransform`]). The playback
//! DTOs ([`CapturedStatsData`], [`CapturedStatsFrame`], [`StatsSnapshotData`],
//! [`StatsSnapshotFrame`]) define the playback-facing serialized shapes.
//!
//! For *timeline*-oriented exports (cumulative stats over time), see the
//! collectors in [`crate::stats::timeline`]
//! ([`StatsTimelineEventCollector`](crate::StatsTimelineEventCollector) and
//! [`StatsTimelineCollector`](crate::StatsTimelineCollector)).

mod builtins;
mod collector;
mod playback;
mod types;

pub use builtins::{
    builtin_analysis_node_json, builtin_analysis_nodes_json, builtin_stats_module_config_json,
    builtin_stats_module_frame_json, builtin_stats_module_json, builtin_stats_module_names,
    default_stats_module_names,
};
pub use collector::{
    FrameTransform, IdentityFrameTransform, ModuleFrameTransform, StatsCollector,
    builtin_stats_graph_snapshot_json,
};
pub use playback::{CapturedStatsData, CapturedStatsFrame, StatsSnapshotData, StatsSnapshotFrame};
pub use types::CollectedStats;
