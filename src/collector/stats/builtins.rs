use std::collections::HashMap;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::stats::analysis_graph::{
    builtin_analysis_node_names, AnalysisGraph, StatsTimelineEventsState, StatsTimelineFrameState,
};
use crate::*;
use boxcars::{Quaternion, RigidBody, Vector3f};

use super::types::serialize_to_json_value;

#[path = "builtins_analysis_json.rs"]
mod builtins_analysis_json;
#[path = "builtins_exports.rs"]
mod builtins_exports;
#[path = "builtins_graph_state.rs"]
mod builtins_graph_state;
#[path = "builtins_module_json.rs"]
mod builtins_module_json;
#[path = "builtins_names.rs"]
mod builtins_names;
#[path = "builtins_snapshot_config.rs"]
mod builtins_snapshot_config;
#[path = "builtins_snapshot_frame.rs"]
mod builtins_snapshot_frame;

pub use builtins_analysis_json::{builtin_analysis_node_json, builtin_analysis_nodes_json};
pub(crate) use builtins_exports::*;
pub(crate) use builtins_graph_state::graph_state;
pub(crate) use builtins_module_json::builtin_module_json;
pub use builtins_module_json::builtin_stats_module_json;
pub use builtins_names::builtin_stats_module_names;
pub(crate) use builtins_snapshot_config::builtin_snapshot_config_json;
pub use builtins_snapshot_config::builtin_stats_module_config_json;
pub(crate) use builtins_snapshot_frame::builtin_snapshot_frame_json;
pub use builtins_snapshot_frame::builtin_stats_module_frame_json;
