use super::*;

fn build_replay_annotations(path: &CStr) -> SubtrActorResult<SaReplayAnnotations> {
    let path = path.to_str().map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "invalid replay path utf-8: {error}"
        )))
    })?;
    let bytes = std::fs::read(path).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "could not read replay file {path}: {error}"
        )))
    })?;
    let replay = ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(|error| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
                "could not parse replay file {path}: {error}"
            )))
        })?;
    let timeline =
        StatsTimelineEventCollector::new().get_replay_stats_timeline_scaffold(&replay)?;
    let events = replay_annotations_from_timeline(&timeline.replay_meta, &timeline.events);
    Ok(SaReplayAnnotations {
        events,
        cursor: 0,
        last_poll_time: 0.0,
        initialized: false,
    })
}

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

/// Drains annotation events whose normal replay-processing timestamp has been
/// reached by the supplied replay playback time.
///
/// The cursor resets automatically after seeking backwards. Events are copied
/// into `out_events` and the return value is the number of copied events.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_poll_replay_annotations(
    annotations: *mut SaReplayAnnotations,
    replay_time: f32,
    out_events: *mut SaMechanicEvent,
    max_events: usize,
) -> usize {
    let Some(annotations) = (unsafe { annotations.as_mut() }) else {
        return 0;
    };
    if max_events == 0 || out_events.is_null() {
        return 0;
    }

    const SEEK_BACK_RESET_SECONDS: f32 = 0.25;
    const LOOKBACK_SECONDS: f32 = 0.20;
    const LOOKAHEAD_SECONDS: f32 = 0.05;

    if !annotations.initialized
        || replay_time + SEEK_BACK_RESET_SECONDS < annotations.last_poll_time
    {
        let restart_time = (replay_time - LOOKBACK_SECONDS).max(0.0);
        annotations.cursor = annotations
            .events
            .partition_point(|event| event.time < restart_time);
        annotations.initialized = true;
    }
    annotations.last_poll_time = replay_time;

    let max_time = replay_time + LOOKAHEAD_SECONDS;
    let mut copied = 0;
    while annotations.cursor < annotations.events.len() && copied < max_events {
        let event = annotations.events[annotations.cursor];
        if event.time > max_time {
            break;
        }
        unsafe {
            out_events.add(copied).write(event);
        }
        annotations.cursor += 1;
        copied += 1;
    }
    copied
}
