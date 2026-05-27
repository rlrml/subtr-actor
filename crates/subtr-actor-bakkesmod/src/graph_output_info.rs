use super::*;

pub(crate) fn live_analysis_graph() -> AnalysisGraph {
    graph_with_all_analysis_nodes()
}

pub(crate) fn serialize_graph_info(graph: &mut AnalysisGraph) -> Vec<u8> {
    let dag = graph.render_ascii_dag().unwrap_or_default();
    let node_names = graph.node_names().collect::<Vec<_>>();
    let callable_analysis_node_names = callable_analysis_node_names_for_graph(graph);
    serde_json::to_vec(&serde_json::json!({
        "builtin_analysis_node_names": builtin_analysis_node_names(),
        "builtin_analysis_node_aliases": builtin_analysis_node_aliases(),
        "callable_analysis_node_names": callable_analysis_node_names,
        "builtin_stats_module_names": builtin_stats_module_names(),
        "graph_output_names": LIVE_GRAPH_OUTPUT_NAMES,
        "graph_event_field_names": LIVE_GRAPH_EVENT_FIELD_NAMES,
        "required_graph_event_field_names": REQUIRED_GRAPH_EVENT_FIELD_NAMES,
        "event_history_field_names": LIVE_EVENT_HISTORY_FIELD_NAMES,
        "required_event_history_field_names": REQUIRED_EVENT_HISTORY_FIELD_NAMES,
        "node_names": node_names,
        "dag": dag,
    }))
    .unwrap_or_default()
}
