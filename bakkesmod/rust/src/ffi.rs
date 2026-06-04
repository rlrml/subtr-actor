use super::*;

pub(crate) fn refresh_timeline_graph_state(engine: &mut SaEngine) {
    let Some(events) = engine
        .graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
    else {
        return;
    };
    push_drainable_events_from_timeline(
        &mut engine.pending_events,
        &mut engine.emitted_mechanic_ids,
        &mut engine.pending_team_events,
        &mut engine.emitted_team_event_ids,
        &mut engine.pending_goal_context_events,
        &mut engine.emitted_goal_context_ids,
        &events,
    );
    if let Some(frame) = current_timeline_frame(&engine.graph) {
        record_timeline_frame(&mut engine.timeline_frames, frame.clone());
    }
}

/// Creates an opaque live-analysis engine.
///
/// The caller owns the returned pointer and must free it with
/// `subtr_actor_bakkesmod_engine_destroy`.
#[no_mangle]
pub extern "C" fn subtr_actor_bakkesmod_engine_create() -> *mut SaEngine {
    Box::into_raw(Box::new(SaEngine::default()))
}

pub(crate) fn build_replay_annotations(path: &CStr) -> SubtrActorResult<SaReplayAnnotations> {
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
    let score_timeline = StatsTimelineCollector::new().get_legacy_replay_stats_timeline(&replay)?;
    let events = replay_annotations_from_timeline(&timeline.replay_meta, &timeline.events);
    let (player_names, players) = replay_annotation_players(&timeline.replay_meta);
    Ok(SaReplayAnnotations {
        events,
        frames: score_timeline.frames,
        players,
        _player_names: player_names,
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

/// Returns the number of replay players available for annotation labels.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotation_player_count(
    annotations: *const SaReplayAnnotations,
) -> usize {
    unsafe { annotations.as_ref() }
        .map(|annotations| annotations.players.len())
        .unwrap_or(0)
}

/// Copies replay player metadata into `out_players`.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_replay_annotation_players(
    annotations: *const SaReplayAnnotations,
    out_players: *mut SaReplayPlayerInfo,
    max_players: usize,
) -> usize {
    let Some(annotations) = (unsafe { annotations.as_ref() }) else {
        return 0;
    };
    if max_players == 0 || out_players.is_null() {
        return 0;
    }
    let count = annotations.players.len().min(max_players);
    unsafe {
        ptr::copy_nonoverlapping(annotations.players.as_ptr(), out_players, count);
    }
    count
}

/// Copies replay players and current-frame core stats for replay playback.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_replay_annotation_frame_players(
    annotations: *const SaReplayAnnotations,
    replay_time: f32,
    out_players: *mut SaPlayerFrame,
    max_players: usize,
) -> usize {
    let Some(annotations) = (unsafe { annotations.as_ref() }) else {
        return 0;
    };
    if max_players == 0 || out_players.is_null() {
        return 0;
    }
    let Some(frame) = replay_annotation_frame_at_time(annotations, replay_time) else {
        return 0;
    };

    let count = frame.players.len().min(max_players);
    for (index, player) in frame.players.iter().take(count).enumerate() {
        let player_info = annotations.players.get(index);
        let player_frame = SaPlayerFrame {
            player_index: player_info
                .map(|info| info.player_index)
                .unwrap_or(index as u32),
            player_name: player_info.map(|info| info.name).unwrap_or(ptr::null()),
            is_team_0: player.is_team_0 as u8,
            has_match_stats: 1,
            match_goals: player.core.goals,
            match_assists: player.core.assists,
            match_saves: player.core.saves,
            match_shots: player.core.shots,
            match_score: player.core.score,
            ..SaPlayerFrame::default()
        };
        unsafe {
            *out_players.add(index) = player_frame;
        }
    }
    count
}

pub(crate) fn serialize_replay_annotation_frame(
    annotations: &SaReplayAnnotations,
    replay_time: f32,
) -> Option<Vec<u8>> {
    replay_annotation_frame_at_time(annotations, replay_time)
        .and_then(|frame| serde_json::to_vec(frame).ok())
}

/// Returns the UTF-8 byte length of the replay stats frame at `replay_time`.
///
/// The JSON payload is a `ReplayStatsFrame` from the preprocessed replay
/// timeline. It is the replay-mode counterpart of
/// `subtr_actor_bakkesmod_frame_json_len`.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotation_frame_json_len(
    annotations: *const SaReplayAnnotations,
    replay_time: f32,
) -> usize {
    annotations
        .as_ref()
        .and_then(|annotations| serialize_replay_annotation_frame(annotations, replay_time))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

/// Writes the replay stats frame at `replay_time` into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_replay_annotation_frame_json_len` first to size the
/// destination buffer.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_replay_annotation_frame_json(
    annotations: *const SaReplayAnnotations,
    replay_time: f32,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(annotations) = annotations.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let Some(bytes) = serialize_replay_annotation_frame(annotations, replay_time) else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

/// Returns the scoreboard value for the latest processed replay frame at or before
/// `replay_time`.
#[no_mangle]
pub unsafe extern "C" fn subtr_actor_bakkesmod_replay_annotation_score_at_time(
    annotations: *const SaReplayAnnotations,
    replay_time: f32,
    out_score: *mut SaReplayScore,
) -> i32 {
    let Some(annotations) = (unsafe { annotations.as_ref() }) else {
        return -1;
    };
    if out_score.is_null() {
        return -1;
    }
    let Some(frame) = replay_annotation_frame_at_time(annotations, replay_time) else {
        return -2;
    };

    unsafe {
        *out_score = SaReplayScore {
            team_zero_score: frame.team_zero.core.goals,
            has_team_zero_score: 1,
            team_one_score: frame.team_one.core.goals,
            has_team_one_score: 1,
        };
    }
    0
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

#[no_mangle]
/// Destroys an engine allocated by `subtr_actor_bakkesmod_engine_create`.
///
/// # Safety
///
/// `engine` must either be null or a pointer returned by
/// `subtr_actor_bakkesmod_engine_create` that has not already been destroyed.
pub unsafe extern "C" fn subtr_actor_bakkesmod_engine_destroy(engine: *mut SaEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

#[no_mangle]
/// Resets an existing engine to its initial state.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_engine_reset(engine: *mut SaEngine) {
    if let Some(engine) = engine.as_mut() {
        *engine = SaEngine::default();
    }
}

#[no_mangle]
/// Finishes live graph evaluation and refreshes exported graph views.
///
/// This mirrors replay collectors' end-of-replay `AnalysisGraph::finish` call,
/// allowing delayed calculators to flush active state before a live engine is
/// reset or destroyed.
///
/// Returns `0` on success, `-1` for an invalid engine pointer, and `-2` if graph
/// finalization fails.
///
/// # Safety
///
/// `engine` must be a valid engine pointer.
pub unsafe extern "C" fn subtr_actor_bakkesmod_finish(engine: *mut SaEngine) -> i32 {
    let Some(engine) = engine.as_mut() else {
        return -1;
    };
    if !engine.live_replay_meta_initialized {
        return 0;
    }
    if engine.live_graph_finished {
        return 0;
    }
    if engine.graph.finish().is_err() {
        return -2;
    }
    refresh_timeline_graph_state(engine);
    engine.live_graph_finished = true;
    0
}

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
    engine.live_graph_finished = false;
    refresh_timeline_graph_state(engine);
    0
}

#[no_mangle]
/// Returns the number of pending events currently buffered by the engine.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_pending_event_count(
    engine: *const SaEngine,
) -> usize {
    engine
        .as_ref()
        .map(|engine| engine.pending_events.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Returns the number of pending team-owned events currently buffered by the engine.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_pending_team_event_count(
    engine: *const SaEngine,
) -> usize {
    engine
        .as_ref()
        .map(|engine| engine.pending_team_events.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Returns the number of pending goal-context events currently buffered by the engine.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_pending_goal_context_event_count(
    engine: *const SaEngine,
) -> usize {
    engine
        .as_ref()
        .map(|engine| engine.pending_goal_context_events.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Returns the UTF-8 byte length of a decoded stats-player config JSON payload.
///
/// Accepts the compressed base64url `cfg` value emitted by the web stats
/// evaluation player. Raw JSON is accepted as a compatibility fallback.
///
/// # Safety
///
/// `encoded_config` must either be null or point to a valid null-terminated
/// UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_decoded_stats_player_config_json_len(
    encoded_config: *const c_char,
) -> usize {
    if encoded_config.is_null() {
        return 0;
    }
    let encoded_config = unsafe { CStr::from_ptr(encoded_config) };
    decode_stats_player_config_json(encoded_config)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes a decoded stats-player config JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_decoded_stats_player_config_json_len` first to size
/// the destination buffer.
///
/// # Safety
///
/// `encoded_config` must point to a valid null-terminated UTF-8 string.
/// `out_bytes` must point to writable storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_decoded_stats_player_config_json(
    encoded_config: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    if encoded_config.is_null() || out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let encoded_config = unsafe { CStr::from_ptr(encoded_config) };
    let Some(bytes) = decode_stats_player_config_json(encoded_config) else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the byte length of the compressed base64url stats-player cfg value.
///
/// The output format matches the web stats evaluation player's `cfg` payload:
/// raw deflate of UTF-8 JSON, encoded as unpadded base64url.
///
/// # Safety
///
/// `json_config` must either be null or point to a valid null-terminated UTF-8
/// JSON string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_encoded_stats_player_config_len(
    json_config: *const c_char,
) -> usize {
    if json_config.is_null() {
        return 0;
    }
    let json_config = unsafe { CStr::from_ptr(json_config) };
    encode_stats_player_config_json(json_config)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the compressed base64url stats-player cfg value into caller storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_encoded_stats_player_config_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `json_config` must point to a valid null-terminated UTF-8 JSON string.
/// `out_bytes` must point to writable storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_encoded_stats_player_config(
    json_config: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    if json_config.is_null() || out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let json_config = unsafe { CStr::from_ptr(json_config) };
    let Some(bytes) = encode_stats_player_config_json(json_config) else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}
