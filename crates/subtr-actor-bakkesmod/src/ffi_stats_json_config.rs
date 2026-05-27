use super::*;

#[no_mangle]
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
    serialize_named_stats_module_config(engine, module_name).len()
}

#[no_mangle]
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
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = serialize_named_stats_module_config(engine, module_name);
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}
