//! C ABI exported to the BakkesMod state-export plugin.
//!
//! Conventions (mirroring `subtr-actor-bakkesmod` / `replay-to-training`):
//! * `SeEngine` is an opaque handle owned by the caller and freed with
//!   [`state_export_engine_destroy`], which shuts the WebSocket server down
//!   and **must** run before the hosting DLL unloads.
//! * String outputs come as `..._len` / `..._write_...` pairs: the `len`
//!   function returns the UTF-8 byte length (no NUL), the `write` function
//!   copies up to `max_bytes` bytes into the caller's buffer and returns the
//!   number of bytes written.
//! * Fallible operations return `0` on success, `-1` for invalid pointers,
//!   and `-2` for operation failures whose message is retrievable through
//!   the engine last-error functions.

use super::*;

pub(crate) const DEFAULT_SERVER_NAME: &str = "subtr-actor-state-export";

/// Opaque engine handle exposed through the C ABI: it owns the WebSocket
/// export server plus the last-error message and the host-supplied match
/// context (re-applied across restarts).
pub struct SeEngine {
    pub(crate) server: Option<ServerHandle>,
    pub(crate) last_error: String,
    pub(crate) match_context: Option<LiveMatchContext>,
}

pub(crate) unsafe fn raw_ref<'a, T>(value: *const T) -> Option<&'a T> {
    // SAFETY: The caller guarantees that any non-null pointer is valid for
    // shared access for the returned lifetime.
    unsafe { value.as_ref() }
}

pub(crate) unsafe fn raw_mut<'a, T>(value: *mut T) -> Option<&'a mut T> {
    // SAFETY: The caller guarantees that any non-null pointer is valid for
    // unique mutable access for the returned lifetime.
    unsafe { value.as_mut() }
}

/// Copies up to `max_bytes` of `text` into `out_bytes`, returning the number
/// of bytes written (no NUL terminator; pair with the `_len` functions).
pub(crate) unsafe fn write_text(text: &str, out_bytes: *mut u8, max_bytes: usize) -> usize {
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let count = bytes.len().min(max_bytes);
    // SAFETY: The caller guarantees `out_bytes` points to writable storage
    // for at least `max_bytes` bytes; `count` is bounded by `max_bytes`.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count) };
    count
}

/// Builds a server config from the ABI config. A null `config` selects the
/// documented plugin defaults, including the recommended
/// [`DEFAULT_STATE_EXPORT_PORT`]; a non-null config keeps `port == 0` as
/// "bind an ephemeral port".
pub(crate) unsafe fn server_config_from_abi(config: *const SeConfig) -> LiveExportServerConfig {
    let defaults = LiveExportServerConfig::default();
    // SAFETY: The caller guarantees `config` is null or valid for reads.
    let Some(config) = (unsafe { raw_ref(config) }) else {
        return LiveExportServerConfig {
            port: DEFAULT_STATE_EXPORT_PORT,
            server_name: DEFAULT_SERVER_NAME.to_owned(),
            ..defaults
        };
    };
    LiveExportServerConfig {
        bind_addr: if config.bind_any_interface != 0 {
            "0.0.0.0".to_owned()
        } else {
            "127.0.0.1".to_owned()
        },
        port: config.port,
        max_ingest_frames: if config.max_queued_frames == 0 {
            defaults.max_ingest_frames
        } else {
            config.max_queued_frames as usize
        },
        max_client_queue: if config.max_client_queue == 0 {
            defaults.max_client_queue
        } else {
            config.max_client_queue as usize
        },
        // SAFETY: The caller guarantees `server_name` is null or a valid
        // null-terminated C string.
        server_name: unsafe { raw_c_string(config.server_name) }
            .unwrap_or_else(|| DEFAULT_SERVER_NAME.to_owned()),
        heartbeat_interval: defaults.heartbeat_interval,
    }
}

pub(crate) fn start_server(engine: &mut SeEngine, config: LiveExportServerConfig) -> bool {
    match LiveExportServer::start(config) {
        Ok(handle) => {
            if let Some(context) = engine.match_context.clone() {
                handle.set_match_context(context);
            }
            engine.server = Some(handle);
            engine.last_error.clear();
            true
        }
        Err(error) => {
            engine.server = None;
            engine.last_error = format!("failed to start export server: {error}");
            false
        }
    }
}

/// Creates an opaque state-export engine and starts its WebSocket server.
///
/// Never returns null: on bind failure the engine is returned in the error
/// state (`state == SE_STATE_ERROR`) so the message is readable through the
/// last-error functions. A null `config` selects the documented defaults,
/// including the recommended default port.
///
/// The caller owns the returned pointer and must free it with
/// `state_export_engine_destroy` **before the hosting DLL unloads**.
///
/// # Safety
///
/// `config` must be null or point to a valid `SeConfig` whose `server_name`
/// is null or a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_engine_create(config: *const SeConfig) -> *mut SeEngine {
    let mut engine = SeEngine {
        server: None,
        last_error: String::new(),
        match_context: None,
    };
    // SAFETY: Forwarding the caller's config validity guarantee.
    let config = unsafe { server_config_from_abi(config) };
    start_server(&mut engine, config);
    Box::into_raw(Box::new(engine))
}

/// Shuts the export server down (joining every server thread) and destroys
/// the engine. Must run before the hosting DLL unloads.
///
/// # Safety
///
/// `engine` must be null or a pointer returned by
/// `state_export_engine_create` that has not already been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_engine_destroy(engine: *mut SeEngine) {
    if engine.is_null() {
        return;
    }
    // SAFETY: The caller guarantees `engine` came from Box::into_raw and has
    // not already been freed. ServerHandle::drop shuts the server down.
    drop(unsafe { Box::from_raw(engine) });
}

/// Stops the running server (if any) and starts a fresh one with `config`
/// (settings-window Apply). The held match context is re-applied to the new
/// server. Returns `0` on success, `-1` for a null engine, and `-2` when the
/// new server fails to start (last-error set; the engine is left stopped in
/// the error state).
///
/// # Safety
///
/// `engine` must be a valid engine pointer; `config` as in
/// `state_export_engine_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_engine_restart(
    engine: *mut SeEngine,
    config: *const SeConfig,
) -> i32 {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return -1;
    };
    if let Some(server) = engine.server.take() {
        server.shutdown();
    }
    // SAFETY: Forwarding the caller's config validity guarantee.
    let config = unsafe { server_config_from_abi(config) };
    if start_server(engine, config) { 0 } else { -2 }
}

/// Converts one sampled frame to the owned live model and enqueues it for
/// broadcast. Never blocks beyond a short mutex hold (the server's ingest
/// queue is bounded with drop-oldest semantics).
///
/// Returns `0` on success, `-1` for invalid pointers or malformed arrays
/// (null pointer with nonzero count), and `-2` when no server is running.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `frame` must point to a valid
/// `SeFrame`; every non-null array in it must hold at least its declared
/// count of elements, and every non-null string must be null-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_push_frame(
    engine: *mut SeEngine,
    frame: *const SeFrame,
) -> i32 {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return -1;
    };
    // SAFETY: Forwarding the caller's frame validity guarantee.
    let Some(frame) = (unsafe { raw_ref(frame) }) else {
        return -1;
    };
    // SAFETY: Forwarding the caller's frame validity guarantee.
    let Ok(live_frame) = (unsafe { live_frame_from_abi(frame) }) else {
        return -1;
    };
    let Some(server) = engine.server.as_ref() else {
        engine.last_error = "push_frame: export server is not running".to_owned();
        return -2;
    };
    server.push_frame(live_frame);
    0
}

/// Sets (or, with a null `context`, clears) the match-level context merged
/// into the broadcast match meta. The context is held on the engine and
/// survives `state_export_engine_restart`; a mid-match change is broadcast
/// as a roster update. Returns `0` on success, `-1` for a null engine.
///
/// # Safety
///
/// `engine` must be a valid engine pointer; `context` must be null or point
/// to a valid `SeMatchContext` whose strings are null or null-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_set_match_context(
    engine: *mut SeEngine,
    context: *const SeMatchContext,
) -> i32 {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return -1;
    };
    // SAFETY: Forwarding the caller's context validity guarantee.
    let context = unsafe { raw_ref(context) }
        // SAFETY: Forwarding the caller's context validity guarantee.
        .map(|context| unsafe { live_match_context(context) })
        .unwrap_or_default();
    engine.match_context = Some(context.clone());
    if let Some(server) = engine.server.as_ref() {
        server.set_match_context(context);
    }
    0
}

/// Broadcasts `MatchEnd` (if a match was live) and resets the export stream
/// (event history, roster, and match context) for the next match. Returns
/// `0` on success, `-1` for a null engine, and `-2` when no server is
/// running.
///
/// # Safety
///
/// `engine` must be a valid engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_notify_match_end(engine: *mut SeEngine) -> i32 {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_mut(engine) }) else {
        return -1;
    };
    // The server clears its broadcast context on MatchEnd; drop the engine's
    // held copy too so a restart does not resurrect the ended match's context.
    engine.match_context = None;
    let Some(server) = engine.server.as_ref() else {
        engine.last_error = "notify_match_end: export server is not running".to_owned();
        return -2;
    };
    server.match_end();
    0
}

/// Writes the engine status into `out_status`. Cheap (atomic reads only), so
/// the C++ tick can poll it every frame, e.g. to gate sampling on
/// `client_count`. Returns `0` on success, `-1` for null pointers.
///
/// # Safety
///
/// `engine` must be a valid engine pointer; `out_status` must point to
/// writable storage for one `SeStatus`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_status(
    engine: *const SeEngine,
    out_status: *mut SeStatus,
) -> i32 {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return -1;
    };
    if out_status.is_null() {
        return -1;
    }
    let status = match engine.server.as_ref() {
        Some(server) => {
            let stats = server.stats();
            SeStatus {
                state: SE_STATE_LISTENING,
                client_count: stats.clients as u32,
                port: server.local_addr().port(),
                frames_sent: stats.frames_sent,
                frames_dropped: stats.frames_dropped,
            }
        }
        None => SeStatus {
            state: if engine.last_error.is_empty() {
                SE_STATE_STOPPED
            } else {
                SE_STATE_ERROR
            },
            ..SeStatus::default()
        },
    };
    // SAFETY: `out_status` is non-null and the caller guarantees it is valid
    // for one write.
    unsafe { out_status.write(status) };
    0
}

/// Returns the UTF-8 byte length of the engine's last error message (0 when
/// there is none or `engine` is null).
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_last_error_len(engine: *const SeEngine) -> usize {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    unsafe { raw_ref(engine) }
        .map(|engine| engine.last_error.len())
        .unwrap_or(0)
}

/// Copies the engine's last error message into `out_bytes` (up to
/// `max_bytes`, no NUL); returns bytes written.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer; `out_bytes` must be null
/// or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_write_last_error(
    engine: *const SeEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    // SAFETY: Forwarding the caller's engine validity guarantee.
    let Some(engine) = (unsafe { raw_ref(engine) }) else {
        return 0;
    };
    // SAFETY: Forwarding the caller's buffer validity guarantee.
    unsafe { write_text(&engine.last_error, out_bytes, max_bytes) }
}

pub(crate) fn build_info() -> String {
    format!(
        "state_export {} build={} dirty={} commit_date={}",
        env!("CARGO_PKG_VERSION"),
        env!("STATE_EXPORT_GIT_HASH"),
        env!("STATE_EXPORT_GIT_DIRTY"),
        env!("STATE_EXPORT_COMMIT_DATE"),
    )
}

/// Returns the UTF-8 byte length of the build identifier string.
#[unsafe(no_mangle)]
pub extern "C" fn state_export_build_info_len() -> usize {
    build_info().len()
}

/// Copies the build identifier ("state_export <version> build=<hash>
/// dirty=<0|1> commit_date=<date>") into `out_bytes` (up to `max_bytes`, no
/// NUL); returns bytes written.
///
/// # Safety
///
/// `out_bytes` must be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn state_export_write_build_info(
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    // SAFETY: Forwarding the caller's buffer validity guarantee.
    unsafe { write_text(&build_info(), out_bytes, max_bytes) }
}
