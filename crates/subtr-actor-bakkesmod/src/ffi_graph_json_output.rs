use super::*;

#[no_mangle]
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

#[no_mangle]
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
