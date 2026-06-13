use super::*;

unsafe fn live_graph_output_len(engine: *const SaEngine, output_name: &str) -> usize {
    unsafe { raw_ref(engine) }
        .and_then(|engine| serialize_live_graph_output(engine, output_name))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

unsafe fn write_live_graph_output(
    engine: *const SaEngine,
    output_name: &str,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return 0;
    };
    let Some(bytes) = serialize_live_graph_output(engine, output_name) else {
        return 0;
    };
    unsafe { copy_to_raw(&bytes, out_bytes, max_bytes) }
}

unsafe fn write_bytes(bytes: &[u8], out_bytes: *mut u8, max_bytes: usize) -> usize {
    unsafe { copy_to_raw(bytes, out_bytes, max_bytes) }
}

unsafe fn drain_pending<T: Copy>(
    pending: &mut Vec<T>,
    out_events: *mut T,
    max_events: usize,
) -> usize {
    let count = unsafe { copy_to_raw(pending, out_events, max_events) };
    pending.drain(..count);
    count
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the current serialized graph event bundle.
///
/// The JSON payload is a `ReplayStatsTimelineEvents` value produced by the live
/// analysis graph after the most recent successful frame.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_events_json_len(engine: *const SaEngine) -> usize {
    unsafe { live_graph_output_len(engine, "events") }
}

#[unsafe(no_mangle)]
/// Writes the current serialized graph event bundle into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_events_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_events_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_live_graph_output(engine, "events", out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the current serialized graph frame snapshot.
///
/// The JSON payload is a `ReplayStatsFrame` value produced by the live analysis
/// graph after the most recent successful frame.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_frame_json_len(engine: *const SaEngine) -> usize {
    unsafe { live_graph_output_len(engine, "frame") }
}

#[unsafe(no_mangle)]
/// Writes the current serialized graph frame snapshot into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_frame_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_frame_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_live_graph_output(engine, "frame", out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the current serialized live stats timeline.
///
/// The JSON payload is a `ReplayStatsTimeline` value produced by the live
/// analysis graph. It contains the graph config, live replay metadata, all
/// timeline event families, and every frame snapshot observed by this engine.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_timeline_json_len(engine: *const SaEngine) -> usize {
    unsafe { live_graph_output_len(engine, "timeline") }
}

#[unsafe(no_mangle)]
/// Writes the current serialized live stats timeline into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_timeline_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_timeline_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_live_graph_output(engine, "timeline", out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the current serialized live stats snapshot.
///
/// The JSON payload exposes the same builtin stats module surface as
/// `StatsCollector`: selected module names, snapshot config, aggregate module
/// JSON, and the current module-keyed frame snapshot when replay metadata and
/// frame state are available.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_json_len(engine: *const SaEngine) -> usize {
    unsafe { live_graph_output_len(engine, "stats") }
}

#[unsafe(no_mangle)]
/// Writes the current serialized live stats snapshot into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_live_graph_output(engine, "stats", out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of one named builtin stats module JSON payload.
///
/// `module_name` must be one of the UTF-8 names reported by
/// `builtin_stats_module_names` in graph info JSON.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    unsafe { serialize_named_stats_module(engine, module_name).len() }
}

#[unsafe(no_mangle)]
/// Writes one named builtin stats module JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = unsafe { serialize_named_stats_module(engine, module_name) };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of one named builtin stats module frame JSON payload.
///
/// Known modules with no per-frame snapshot return JSON `null`; unknown modules
/// and invalid inputs return length `0`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_frame_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    unsafe { serialize_named_stats_module_frame(engine, module_name).len() }
}

#[unsafe(no_mangle)]
/// Writes one named builtin stats module frame JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_frame_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_frame_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = unsafe { serialize_named_stats_module_frame(engine, module_name) };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of one named builtin stats module config JSON payload.
///
/// Known modules with no snapshot config return JSON `null`; unknown modules and
/// invalid inputs return length `0`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_config_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    unsafe { serialize_named_stats_module_config(engine, module_name).len() }
}

#[unsafe(no_mangle)]
/// Writes one named builtin stats module config JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_config_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_config_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = unsafe { serialize_named_stats_module_config(engine, module_name) };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of one named live graph output JSON payload.
///
/// `output_name` must be one of `events`, `frame`, `timeline`, `stats`,
/// `analysis_nodes`, `event_history`, or `graph_info`, which are also reported
/// by graph info JSON.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `output_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_graph_output_json_len(
    engine: *const SaEngine,
    output_name: *const c_char,
) -> usize {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return 0;
    };
    let Some(output_name) = (unsafe { c_string_arg(output_name) }) else {
        return 0;
    };
    serialize_live_graph_output(engine, &output_name)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// Writes one named live graph output JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_graph_output_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `output_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_graph_output_json(
    engine: *const SaEngine,
    output_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return 0;
    };
    let Some(output_name) = (unsafe { c_string_arg(output_name) }) else {
        return 0;
    };
    let Some(bytes) = serialize_live_graph_output(engine, &output_name) else {
        return 0;
    };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of one named live analysis-node JSON payload.
///
/// `node_name` must be one of the names reported by
/// `subtr_actor_bakkesmod_analysis_node_names_json_len`. Calculator nodes use
/// the same graph-backed payloads as stats modules; signal/state nodes use
/// structured snapshots of their current graph state.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `node_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_analysis_node_json_len(
    engine: *const SaEngine,
    node_name: *const c_char,
) -> usize {
    unsafe { serialize_named_analysis_node(engine, node_name).len() }
}

#[unsafe(no_mangle)]
/// Writes one named live analysis-node JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_analysis_node_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `node_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_analysis_node_json(
    engine: *const SaEngine,
    node_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = unsafe { serialize_named_analysis_node(engine, node_name) };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the callable analysis-node name registry.
///
/// The payload is a JSON string array containing every supported name for
/// `subtr_actor_bakkesmod_analysis_node_json_len`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_analysis_node_names_json_len(
    engine: *const SaEngine,
) -> usize {
    unsafe { serialize_analysis_node_names(engine).len() }
}

#[unsafe(no_mangle)]
/// Writes the callable analysis-node name registry into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_analysis_node_names_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_analysis_node_names_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = unsafe { serialize_analysis_node_names(engine) };
    unsafe { write_bytes(&bytes, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Returns the UTF-8 byte length of the serialized live graph metadata.
///
/// The JSON payload includes the builtin analysis-node registry, the actual
/// node names configured in this engine, and an ASCII DAG rendering.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_graph_info_json_len(
    engine: *const SaEngine,
) -> usize {
    unsafe { raw_ref(engine) }
        .map(|engine| engine.graph_info_json.len())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
/// Writes the serialized live graph metadata into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_graph_info_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_graph_info_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return 0;
    };
    unsafe { write_bytes(&engine.graph_info_json, out_bytes, max_bytes) }
}

#[unsafe(no_mangle)]
/// Copies and removes pending events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaMechanicEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_events(
    engine: *mut SaEngine,
    out_events: *mut SaMechanicEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return 0;
    };
    unsafe { drain_pending(&mut engine.pending_events, out_events, max_events) }
}

#[unsafe(no_mangle)]
/// Copies and removes pending team-owned events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaTeamEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_team_events(
    engine: *mut SaEngine,
    out_events: *mut SaTeamEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return 0;
    };
    unsafe { drain_pending(&mut engine.pending_team_events, out_events, max_events) }
}

#[unsafe(no_mangle)]
/// Copies and removes pending goal-context events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaGoalContextEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_goal_context_events(
    engine: *mut SaEngine,
    out_events: *mut SaGoalContextEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return 0;
    };
    unsafe {
        drain_pending(
            &mut engine.pending_goal_context_events,
            out_events,
            max_events,
        )
    }
}
