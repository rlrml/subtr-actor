use super::*;

pub(crate) fn serialize_stats_graph_snapshot(engine: &SaEngine) -> Vec<u8> {
    match builtin_stats_graph_snapshot_json(&engine.graph, engine.live_replay_meta.as_ref()) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn serialize_analysis_nodes_snapshot(engine: &SaEngine) -> Vec<u8> {
    match callable_analysis_nodes_json(&engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn callable_analysis_nodes_json(
    graph: &AnalysisGraph,
) -> SubtrActorResult<serde_json::Value> {
    let mut values = serde_json::Map::new();
    for node_name in callable_analysis_node_names_for_graph(graph) {
        values.insert(
            node_name.clone(),
            builtin_analysis_node_json(&node_name, graph)?,
        );
    }
    Ok(serde_json::Value::Object(values))
}

pub(crate) unsafe fn serialize_named_analysis_node(
    engine: *const SaEngine,
    node_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(node_name) = c_string_arg(node_name) else {
        return Vec::new();
    };
    match builtin_analysis_node_json(&node_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}
