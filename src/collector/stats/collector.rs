use std::marker::PhantomData;

use crate::collector::frame_resolution::StatsFramePersistenceController;
use crate::stats::analysis_graph::AnalysisGraph;

use super::playback::StatsSnapshotFrame;

#[path = "collector_config.rs"]
mod config;
#[path = "collector_constructors.rs"]
mod constructors;
#[path = "collector_frame_snapshots.rs"]
mod frame_snapshots;
#[path = "collector_legacy_outputs.rs"]
mod legacy_outputs;
#[path = "collector_process.rs"]
mod process;
#[path = "collector_process_helpers.rs"]
mod process_helpers;
#[path = "collector_run.rs"]
mod run;
#[path = "collector_selection.rs"]
mod selection;
#[path = "collector_selection_snapshot.rs"]
mod selection_snapshot;
#[path = "collector_snapshot_json.rs"]
mod snapshot_json;
#[path = "collector_transform.rs"]
mod transform;

pub use snapshot_json::builtin_stats_graph_snapshot_json;
pub use transform::{FrameTransform, IdentityFrameTransform, ModuleFrameTransform};

#[derive(Default)]
enum SampleMode {
    #[default]
    Aggregate,
    Timeline,
}

struct BuiltinModuleSelection {
    module_names: Vec<&'static str>,
}

pub struct StatsCollector<T = StatsSnapshotFrame, F = IdentityFrameTransform> {
    modules: BuiltinModuleSelection,
    graph: AnalysisGraph,
    replay_meta: Option<crate::ReplayMeta>,
    last_replay_meta_player_count: Option<usize>,
    frame_transform: F,
    captured_frames: Option<Vec<T>>,
    sample_mode: SampleMode,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_dodge_refreshed_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
    _marker: PhantomData<T>,
}
