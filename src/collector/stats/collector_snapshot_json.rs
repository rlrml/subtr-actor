use serde_json::{Map, Value};

use crate::stats::analysis_graph::AnalysisGraph;
use crate::{FrameInfo, GameplayState, ReplayMeta, SubtrActorResult};

use super::super::types::serialize_to_json_value;
use super::BuiltinModuleSelection;

pub fn builtin_stats_graph_snapshot_json(
    graph: &AnalysisGraph,
    replay_meta: Option<&ReplayMeta>,
) -> SubtrActorResult<Value> {
    let modules = BuiltinModuleSelection::all();
    let frame = if let Some(replay_meta) = replay_meta {
        if graph.state::<FrameInfo>().is_some() && graph.state::<GameplayState>().is_some() {
            serialize_to_json_value(&modules.snapshot_frame(graph, replay_meta)?)?
        } else {
            Value::Null
        }
    } else {
        Value::Null
    };

    let mut payload = Map::new();
    payload.insert(
        "module_names".to_owned(),
        serialize_to_json_value(&modules.module_names)?,
    );
    payload.insert(
        "config".to_owned(),
        Value::Object(modules.snapshot_config_json(graph)?),
    );
    payload.insert(
        "modules".to_owned(),
        Value::Object(modules.modules_json(graph)?),
    );
    payload.insert("frame".to_owned(), frame);
    Ok(Value::Object(payload))
}
