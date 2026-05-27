use super::*;

#[no_mangle]
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

#[no_mangle]
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
    let bytes = serialize_analysis_node_names(engine);
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}
