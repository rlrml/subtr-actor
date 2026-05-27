use super::*;

#[path = "builtins_analysis_json_derived_nodes.rs"]
mod builtins_analysis_json_derived_nodes;
#[path = "builtins_analysis_json_frame_nodes.rs"]
mod builtins_analysis_json_frame_nodes;
#[path = "builtins_analysis_json_physics.rs"]
mod builtins_analysis_json_physics;
#[path = "builtins_analysis_json_players.rs"]
mod builtins_analysis_json_players;
#[path = "builtins_analysis_json_state_nodes.rs"]
mod builtins_analysis_json_state_nodes;

use builtins_analysis_json_derived_nodes::derived_analysis_node_json;
use builtins_analysis_json_frame_nodes::frame_analysis_node_json;
use builtins_analysis_json_physics::*;
use builtins_analysis_json_players::*;
use builtins_analysis_json_state_nodes::*;

pub fn builtin_analysis_node_json(
    node_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    if matches!(node_name, "core" | "match_stats") {
        return builtin_module_json("core", graph);
    }
    if let Some(value) = frame_analysis_node_json(node_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = derived_analysis_node_json(node_name, graph)? {
        return Ok(value);
    }
    if builtin_stats_module_names().contains(&node_name) {
        return builtin_module_json(node_name, graph);
    }

    SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
        node_name.to_owned(),
    ))
}

pub fn builtin_analysis_nodes_json(graph: &AnalysisGraph) -> SubtrActorResult<Value> {
    let mut values = Map::new();
    for node_name in builtin_analysis_node_names() {
        values.insert(
            (*node_name).to_owned(),
            builtin_analysis_node_json(node_name, graph)?,
        );
    }
    Ok(Value::Object(values))
}
