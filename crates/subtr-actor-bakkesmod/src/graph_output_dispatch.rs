use super::*;

pub(crate) fn serialize_live_graph_output(engine: &SaEngine, output_name: &str) -> Option<Vec<u8>> {
    match output_name {
        "events" => current_timeline_events(&engine.graph)
            .map(|events| serde_json::to_vec(&events).unwrap_or_default()),
        "frame" => current_timeline_frame(&engine.graph)
            .map(|frame| serde_json::to_vec(&frame).unwrap_or_default()),
        "timeline" => current_timeline_events(&engine.graph).map(|events| {
            serialize_live_timeline(
                engine.live_replay_meta.as_ref(),
                events,
                engine.timeline_frames.clone(),
            )
        }),
        "stats" => Some(serialize_stats_graph_snapshot(engine)),
        "analysis_nodes" => Some(serialize_analysis_nodes_snapshot(engine)),
        "event_history" => Some(serialize_live_event_history(engine)),
        "graph_info" => Some(engine.graph_info_json.clone()),
        _ => None,
    }
}
