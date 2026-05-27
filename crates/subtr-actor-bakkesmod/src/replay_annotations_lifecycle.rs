use super::replay_annotations_build::build_replay_annotations;
use super::*;

/// Parses a replay file through the normal replay processor and precomputes
/// time-indexed annotation events for replay playback overlays.
///
/// Returns null on failure. The returned handle must be destroyed with
/// `subtr_actor_bakkesmod_replay_annotations_destroy`.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotations_create(
    replay_path: *const c_char,
) -> *mut SaReplayAnnotations {
    if replay_path.is_null() {
        return ptr::null_mut();
    }
    let replay_path = unsafe { CStr::from_ptr(replay_path) };
    match build_replay_annotations(replay_path) {
        Ok(annotations) => Box::into_raw(Box::new(annotations)),
        Err(_) => ptr::null_mut(),
    }
}

/// Destroys a replay annotation handle allocated by
/// `subtr_actor_bakkesmod_replay_annotations_create`.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotations_destroy(
    annotations: *mut SaReplayAnnotations,
) {
    if !annotations.is_null() {
        drop(unsafe { Box::from_raw(annotations) });
    }
}

/// Returns the number of precomputed replay annotation events.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotation_count(
    annotations: *const SaReplayAnnotations,
) -> usize {
    unsafe { annotations.as_ref() }
        .map(|annotations| annotations.events.len())
        .unwrap_or(0)
}
