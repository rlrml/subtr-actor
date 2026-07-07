use super::*;

pub(crate) fn current_timeline_frame(graph: &AnalysisGraph) -> Option<ReplayStatsFrame> {
    graph
        .state::<StatsTimelineFrameState>()
        .and_then(|state| state.frame.clone())
}

pub(crate) fn record_timeline_frame(frames: &mut Vec<ReplayStatsFrame>, frame: ReplayStatsFrame) {
    if let Some(last_frame) = frames.last_mut() {
        if last_frame.frame_number == frame.frame_number {
            *last_frame = frame;
            return;
        }
    }
    frames.push(frame);
}

pub(crate) fn serialize_live_timeline(
    replay_meta: Option<&ReplayMeta>,
    events: ReplayStatsTimelineEvents,
    frames: Vec<ReplayStatsFrame>,
) -> Vec<u8> {
    let Some(replay_meta) = replay_meta else {
        return Vec::new();
    };
    serde_json::to_vec(&ReplayStatsTimeline {
        config: default_stats_timeline_config(),
        replay_meta: replay_meta.clone(),
        events,
        frames,
    })
    .unwrap_or_default()
}

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
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return Vec::new();
    };
    let Some(node_name) = (unsafe { c_string_arg(node_name) }) else {
        return Vec::new();
    };
    match builtin_analysis_node_json(&node_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn callable_analysis_node_names_for_graph(graph: &AnalysisGraph) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.extend(graph.node_names().map(str::to_owned));
    names.extend(
        builtin_analysis_node_names()
            .iter()
            .map(|name| (*name).to_owned()),
    );
    names.into_iter().collect()
}

pub(crate) fn callable_analysis_node_names(engine: &SaEngine) -> Vec<String> {
    callable_analysis_node_names_for_graph(&engine.graph)
}

pub(crate) unsafe fn serialize_analysis_node_names(engine: *const SaEngine) -> Vec<u8> {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return Vec::new();
    };
    serde_json::to_vec(&callable_analysis_node_names(engine)).unwrap_or_default()
}

pub(crate) unsafe fn c_string_arg(value: *const c_char) -> Option<String> {
    unsafe { raw_c_string(value) }
}

pub(crate) unsafe fn serialize_named_stats_module(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return Vec::new();
    };
    let Some(module_name) = (unsafe { c_string_arg(module_name) }) else {
        return Vec::new();
    };
    match builtin_stats_module_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) unsafe fn serialize_named_stats_module_frame(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return Vec::new();
    };
    let Some(module_name) = (unsafe { c_string_arg(module_name) }) else {
        return Vec::new();
    };
    let Some(replay_meta) = engine.live_replay_meta.as_ref() else {
        return Vec::new();
    };
    match builtin_stats_module_frame_json(&module_name, &engine.graph, replay_meta) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) unsafe fn serialize_named_stats_module_config(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return Vec::new();
    };
    let Some(module_name) = (unsafe { c_string_arg(module_name) }) else {
        return Vec::new();
    };
    match builtin_stats_module_config_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn serialize_live_event_history(engine: &SaEngine) -> Vec<u8> {
    let active_demos: Vec<_> = engine
        .live_events
        .active_demos
        .iter()
        .map(|active_demo| {
            serde_json::json!({
                "attacker": &active_demo.sample.attacker,
                "victim": &active_demo.sample.victim,
            })
        })
        .collect();
    serde_json::to_vec(&serde_json::json!({
        "active_demos": active_demos,
        "demo_events": &engine.live_event_history.demo_events,
        "boost_pad_events": &engine.live_event_history.boost_pad_events,
        "touch_events": &engine.live_event_history.touch_events,
        "dodge_refreshed_events": &engine.live_event_history.dodge_refreshed_events,
        "player_stat_events": &engine.live_event_history.player_stat_events,
        "goal_events": &engine.live_event_history.goal_events,
    }))
    .unwrap_or_default()
}

pub(crate) fn current_timeline_events(graph: &AnalysisGraph) -> Option<ReplayStatsTimelineEvents> {
    graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
}

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

pub(crate) fn inflate_stats_player_config_bytes(compressed: &[u8]) -> Option<Vec<u8>> {
    let mut raw_decoder = DeflateDecoder::new(compressed);
    let mut json = Vec::new();
    if raw_decoder.read_to_end(&mut json).is_ok()
        && serde_json::from_slice::<serde_json::Value>(&json).is_ok()
    {
        return Some(json);
    }

    let mut zlib_decoder = ZlibDecoder::new(compressed);
    let mut json = Vec::new();
    if zlib_decoder.read_to_end(&mut json).is_ok()
        && serde_json::from_slice::<serde_json::Value>(&json).is_ok()
    {
        return Some(json);
    }

    None
}

pub(crate) fn decode_stats_player_config_json(value: &CStr) -> Option<Vec<u8>> {
    let value = value.to_str().ok()?.trim();
    if value.starts_with('{') {
        return Some(value.as_bytes().to_vec());
    }

    let compressed = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .ok()?;
    inflate_stats_player_config_bytes(&compressed)
}

pub(crate) fn encode_stats_player_config_json(value: &CStr) -> Option<Vec<u8>> {
    let value = value.to_str().ok()?.trim();
    serde_json::from_str::<serde_json::Value>(value).ok()?;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(value.as_bytes()).ok()?;
    let compressed = encoder.finish().ok()?;
    Some(URL_SAFE_NO_PAD.encode(compressed).into_bytes())
}
