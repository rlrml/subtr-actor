use super::*;

const SEEK_BACK_RESET_SECONDS: f32 = 0.25;
const LOOKBACK_SECONDS: f32 = 0.20;
const LOOKAHEAD_SECONDS: f32 = 0.05;

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

    reset_cursor_after_seek(annotations, replay_time);
    copy_ready_events(annotations, replay_time, out_events, max_events)
}

fn reset_cursor_after_seek(annotations: &mut SaReplayAnnotations, replay_time: f32) {
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
}

fn copy_ready_events(
    annotations: &mut SaReplayAnnotations,
    replay_time: f32,
    out_events: *mut SaMechanicEvent,
    max_events: usize,
) -> usize {
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
