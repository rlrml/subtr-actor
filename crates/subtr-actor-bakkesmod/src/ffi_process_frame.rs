use super::*;

#[no_mangle]
/// Feeds one sampled Rocket League frame into the live mechanics engine.
///
/// Returns `0` on success, `-1` for invalid pointers, and `-2` if detector
/// evaluation fails.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `frame` must point to a valid
/// `SaLiveFrame`; when `player_count` is nonzero, `frame.players` must point to
/// an array containing at least `player_count` `SaPlayerFrame` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_process_frame(
    engine: *mut SaEngine,
    frame: *const SaLiveFrame,
) -> i32 {
    let Some(engine) = engine.as_mut() else {
        return -1;
    };
    let Some(frame) = frame.as_ref() else {
        return -1;
    };
    if frame.players.is_null() && frame.player_count != 0 {
        return -1;
    }

    let players = if frame.player_count == 0 {
        &[]
    } else {
        slice::from_raw_parts(frame.players, frame.player_count)
    };
    if has_duplicate_player_indices(players) {
        return -1;
    }
    let Ok(explicit_events) = frame_event_slices(frame) else {
        return -1;
    };
    if sync_live_replay_meta(engine, players).is_err() {
        return -2;
    }
    let mut live_events = engine.live_events.clone();
    let mut live_event_history = engine.live_event_history.clone();
    let frame_input = frame_input_from_live_state(
        &mut live_events,
        &mut live_event_history,
        engine.live_replay_meta.as_ref(),
        frame,
        players,
        &explicit_events,
    );
    if engine.graph.evaluate_with_state(&frame_input).is_err() {
        return -2;
    }

    engine.live_events = live_events;
    engine.live_event_history = live_event_history;
    refresh_timeline_graph_state(engine);
    0
}
