use super::*;

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
    unsafe {
        engine
            .as_ref()
            .and_then(|engine| serialize_live_graph_output(engine, "events"))
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }

        let Some(bytes) = serialize_live_graph_output(engine, "events") else {
            return 0;
        };
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        engine
            .as_ref()
            .and_then(|engine| serialize_live_graph_output(engine, "frame"))
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }

        let Some(bytes) = serialize_live_graph_output(engine, "frame") else {
            return 0;
        };
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        engine
            .as_ref()
            .and_then(|engine| serialize_live_graph_output(engine, "timeline"))
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }

        let Some(bytes) = serialize_live_graph_output(engine, "timeline") else {
            return 0;
        };
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        engine
            .as_ref()
            .and_then(|engine| serialize_live_graph_output(engine, "stats"))
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }

        let Some(bytes) = serialize_live_graph_output(engine, "stats") else {
            return 0;
        };
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let bytes = serialize_named_stats_module(engine, module_name);
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let bytes = serialize_named_stats_module_frame(engine, module_name);
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let bytes = serialize_named_stats_module_config(engine, module_name);
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        let Some(output_name) = c_string_arg(output_name) else {
            return 0;
        };
        serialize_live_graph_output(engine, &output_name)
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        let Some(output_name) = c_string_arg(output_name) else {
            return 0;
        };
        let Some(bytes) = serialize_live_graph_output(engine, &output_name) else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        let bytes = serialize_named_analysis_node(engine, node_name);
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    serialize_analysis_node_names(engine).len()
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
    unsafe {
        let bytes = serialize_analysis_node_names(engine);
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }
        let count = max_bytes.min(bytes.len());
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        engine
            .as_ref()
            .map(|engine| engine.graph_info_json.len())
            .unwrap_or(0)
    }
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
    unsafe {
        let Some(engine) = engine.as_ref() else {
            return 0;
        };
        if out_bytes.is_null() || max_bytes == 0 {
            return 0;
        }

        let count = max_bytes.min(engine.graph_info_json.len());
        ptr::copy_nonoverlapping(engine.graph_info_json.as_ptr(), out_bytes, count);
        count
    }
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
    unsafe {
        let Some(engine) = engine.as_mut() else {
            return 0;
        };
        if out_events.is_null() || max_events == 0 {
            return 0;
        }

        let count = max_events.min(engine.pending_events.len());
        ptr::copy_nonoverlapping(engine.pending_events.as_ptr(), out_events, count);
        engine.pending_events.drain(..count);
        count
    }
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
    unsafe {
        let Some(engine) = engine.as_mut() else {
            return 0;
        };
        if out_events.is_null() || max_events == 0 {
            return 0;
        }

        let count = max_events.min(engine.pending_team_events.len());
        ptr::copy_nonoverlapping(engine.pending_team_events.as_ptr(), out_events, count);
        engine.pending_team_events.drain(..count);
        count
    }
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
    unsafe {
        let Some(engine) = engine.as_mut() else {
            return 0;
        };
        if out_events.is_null() || max_events == 0 {
            return 0;
        }

        let count = max_events.min(engine.pending_goal_context_events.len());
        ptr::copy_nonoverlapping(
            engine.pending_goal_context_events.as_ptr(),
            out_events,
            count,
        );
        engine.pending_goal_context_events.drain(..count);
        count
    }
}
