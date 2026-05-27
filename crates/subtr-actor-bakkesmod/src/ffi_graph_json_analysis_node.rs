use super::*;

#[no_mangle]
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
    serialize_named_analysis_node(engine, node_name).len()
}

#[no_mangle]
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
    let bytes = serialize_named_analysis_node(engine, node_name);
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}
