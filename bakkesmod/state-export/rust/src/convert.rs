//! repr(C) -> owned conversions, mirroring
//! `bakkesmod/subtr-actor/rust/src/abi_convert.rs`.
//!
//! Every pointer field is validated (null => `None` / empty). Non-finite
//! floats are sanitized *here*, at the ABI boundary, because the export
//! server JSON-encodes frames for JSON clients and `serde_json` refuses
//! NaN/inf (a poisoned float would silently drop the whole message for those
//! clients): required floats collapse to `0.0`, optional floats to absent.

use super::*;

pub(crate) fn finite_f32(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

pub(crate) fn finite_opt_f32(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

pub(crate) fn vec3(value: SeVec3) -> Vector3f {
    Vector3f {
        x: finite_f32(value.x),
        y: finite_f32(value.y),
        z: finite_f32(value.z),
    }
}

pub(crate) fn vec3_array(value: SeVec3) -> [f32; 3] {
    [
        finite_f32(value.x),
        finite_f32(value.y),
        finite_f32(value.z),
    ]
}

pub(crate) fn quat(value: SeQuat) -> Quaternion {
    if [value.x, value.y, value.z, value.w]
        .iter()
        .all(|component| component.is_finite())
    {
        Quaternion {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    } else {
        // A partially-poisoned rotation is meaningless; collapse to identity.
        Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

pub(crate) fn rigid_body(value: SeRigidBody) -> RigidBody {
    RigidBody {
        location: vec3(value.location),
        rotation: quat(value.rotation),
        sleeping: value.sleeping != 0,
        linear_velocity: (value.has_linear_velocity != 0).then_some(vec3(value.linear_velocity)),
        angular_velocity: (value.has_angular_velocity != 0).then_some(vec3(value.angular_velocity)),
    }
}

pub(crate) unsafe fn checked_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    if items.is_null() && count != 0 {
        return Err(());
    }
    if count == 0 {
        Ok(&[])
    } else {
        // SAFETY: The caller guarantees `items` points to at least `count`
        // initialized elements.
        Ok(unsafe { slice::from_raw_parts(items, count) })
    }
}

pub(crate) unsafe fn raw_c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    // SAFETY: The caller guarantees `value` points to a valid null-terminated
    // C string.
    let text = unsafe { CStr::from_ptr(value) }.to_string_lossy();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

pub(crate) unsafe fn player_name(player: &SePlayerFrame) -> Option<String> {
    // SAFETY: Forwarding the caller's C-string validity guarantee.
    unsafe { raw_c_string(player.player_name) }
}

pub(crate) fn player_car_body_id(player: &SePlayerFrame) -> Option<u32> {
    if player.has_car_body_id == 0 {
        return None;
    }
    u32::try_from(player.car_body_id).ok()
}

pub(crate) fn controller_input(input: &SeControllerInput) -> LiveControllerInput {
    LiveControllerInput {
        throttle: finite_f32(input.throttle),
        steer: finite_f32(input.steer),
        pitch: finite_f32(input.pitch),
        yaw: finite_f32(input.yaw),
        roll: finite_f32(input.roll),
        dodge_forward: finite_f32(input.dodge_forward),
        dodge_strafe: finite_f32(input.dodge_strafe),
        handbrake: input.handbrake != 0,
        jump: input.jump != 0,
        activate_boost: input.activate_boost != 0,
        holding_boost: input.holding_boost != 0,
    }
}

/// Converts camera state; a camera with no available part converts to `None`.
pub(crate) fn camera_state(camera: &SeCameraState) -> Option<LiveCameraState> {
    let state = LiveCameraState {
        pitch: (camera.has_pitch != 0).then_some(camera.pitch),
        yaw: (camera.has_yaw != 0).then_some(camera.yaw),
        ball_cam_active: (camera.has_ball_cam != 0).then_some(camera.ball_cam_active != 0),
    };
    (state.pitch.is_some() || state.yaw.is_some() || state.ball_cam_active.is_some())
        .then_some(state)
}

/// Maps an [`SeRemoteId`] to a [`boxcars::RemoteId`], following the
/// `SE_REMOTE_ID_PLATFORM_*` table in `abi.rs`: only platforms whose boxcars
/// identity is lossless from `(platform, online_id | epic_id)` map to `Some`;
/// everything else (none, psynet, playstation, unknown values) converts to
/// `None`, whose fallback identity is `RemoteId::SplitScreen(player_index)`.
pub(crate) unsafe fn remote_id(value: &SeRemoteId) -> Option<RemoteId> {
    match value.platform {
        SE_REMOTE_ID_PLATFORM_STEAM => Some(RemoteId::Steam(value.online_id)),
        // SAFETY: Forwarding the caller's C-string validity guarantee.
        SE_REMOTE_ID_PLATFORM_EPIC => unsafe { raw_c_string(value.epic_id) }.map(RemoteId::Epic),
        SE_REMOTE_ID_PLATFORM_XBOX => Some(RemoteId::Xbox(value.online_id)),
        SE_REMOTE_ID_PLATFORM_SWITCH => Some(RemoteId::Switch(SwitchId {
            online_id: value.online_id,
            unknown1: Vec::new(),
        })),
        SE_REMOTE_ID_PLATFORM_SPLITSCREEN => Some(RemoteId::SplitScreen(value.splitscreen_index)),
        SE_REMOTE_ID_PLATFORM_QQ => Some(RemoteId::QQ(value.online_id)),
        // None, psynet, and playstation ids carry structured payloads that
        // cannot be replicated losslessly from this ABI.
        _ => None,
    }
}

fn live_event_timing(timing: SeEventTiming) -> LiveEventTiming {
    LiveEventTiming {
        frame_and_time: (timing.has_timing != 0)
            .then_some((timing.frame_number, finite_f32(timing.time))),
        seconds_remaining: (timing.has_seconds_remaining != 0).then_some(timing.seconds_remaining),
    }
}

pub(crate) unsafe fn live_player_frame(player: &SePlayerFrame) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index: player.player_index,
        // SAFETY: Forwarding the caller's pointer validity guarantees.
        name: unsafe { player_name(player) },
        // SAFETY: Forwarding the caller's pointer validity guarantees.
        remote_id: unsafe { remote_id(&player.remote_id) },
        is_team_0: player.is_team_0 != 0,
        rigid_body: (player.has_rigid_body != 0).then(|| rigid_body(player.rigid_body)),
        boost_amount: finite_f32(player.boost_amount),
        last_boost_amount: finite_f32(player.last_boost_amount),
        boost_active: player.boost_active,
        jump_active: player.jump_active,
        double_jump_active: player.double_jump_active,
        dodge_active: player.dodge_active,
        powerslide_active: player.powerslide_active != 0,
        input: (player.has_input != 0).then(|| controller_input(&player.input)),
        camera: camera_state(&player.camera),
        dodge_impulse: (player.has_dodge_impulse != 0).then(|| vec3_array(player.dodge_impulse)),
        dodge_torque: (player.has_dodge_torque != 0).then(|| vec3_array(player.dodge_torque)),
        car_body_id: player_car_body_id(player),
        match_stats: (player.has_match_stats != 0).then_some(LiveMatchStats {
            goals: player.match_goals,
            assists: player.match_assists,
            saves: player.match_saves,
            shots: player.match_shots,
            score: player.match_score,
        }),
    }
}

pub(crate) unsafe fn live_player_frames(players: &[SePlayerFrame]) -> Vec<LivePlayerFrame> {
    players
        .iter()
        // SAFETY: Forwarding the caller's pointer validity guarantees.
        .map(|player| unsafe { live_player_frame(player) })
        .collect()
}

fn live_touch_event(event: &SeTouchEvent) -> LiveTouchEvent {
    LiveTouchEvent {
        timing: live_event_timing(event.timing),
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
        is_team_0: event.is_team_0 != 0,
        closest_approach_distance: (event.has_closest_approach_distance != 0)
            .then_some(event.closest_approach_distance)
            .and_then(finite_opt_f32),
    }
}

fn live_dodge_refreshed_event(event: &SeDodgeRefreshedEvent) -> LiveDodgeRefreshedEvent {
    LiveDodgeRefreshedEvent {
        timing: live_event_timing(event.timing),
        player: player_id(event.player_index),
        is_team_0: event.is_team_0 != 0,
        counter_value: event.counter_value,
    }
}

fn live_boost_pad_event(event: &SeBoostPadEvent) -> LiveBoostPadEvent {
    LiveBoostPadEvent {
        timing: live_event_timing(event.timing),
        pad_id: event.pad_id.to_string(),
        kind: match event.kind {
            SeBoostPadEventKind::PickedUp => LiveBoostPadEventKind::PickedUp,
            SeBoostPadEventKind::Available => LiveBoostPadEventKind::Available,
        },
        sequence: event.sequence,
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
    }
}

fn live_goal_event(event: &SeGoalEvent) -> LiveGoalEvent {
    LiveGoalEvent {
        timing: live_event_timing(event.timing),
        scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
        team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
        team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
    }
}

fn live_player_stat_event(event: &SePlayerStatEvent) -> LivePlayerStatEvent {
    LivePlayerStatEvent {
        timing: live_event_timing(event.timing),
        player: player_id(event.player_index),
        is_team_0: event.is_team_0 != 0,
        kind: match event.kind {
            SePlayerStatEventKind::Shot => LivePlayerStatEventKind::Shot,
            SePlayerStatEventKind::Save => LivePlayerStatEventKind::Save,
            SePlayerStatEventKind::Assist => LivePlayerStatEventKind::Assist,
        },
        shot_ball: (event.has_shot_ball != 0).then(|| rigid_body(event.shot_ball)),
        shot_player: (event.has_shot_player != 0).then(|| rigid_body(event.shot_player)),
    }
}

fn live_demolish_event(event: &SeDemolishEvent) -> LiveDemolishEvent {
    LiveDemolishEvent {
        timing: live_event_timing(event.timing),
        attacker: player_id(event.attacker_index),
        victim: player_id(event.victim_index),
        attacker_velocity: vec3(event.attacker_velocity),
        victim_velocity: vec3(event.victim_velocity),
        victim_location: vec3(event.victim_location),
        active_duration_seconds: finite_f32(event.active_duration_seconds),
    }
}

/// Borrowed views of the six explicit event arrays of an [`SeFrame`].
pub(crate) struct SeFrameEventSlices<'a> {
    pub touches: &'a [SeTouchEvent],
    pub dodge_refreshes: &'a [SeDodgeRefreshedEvent],
    pub boost_pad_events: &'a [SeBoostPadEvent],
    pub goals: &'a [SeGoalEvent],
    pub player_stat_events: &'a [SePlayerStatEvent],
    pub demolishes: &'a [SeDemolishEvent],
}

pub(crate) unsafe fn frame_event_slices(frame: &SeFrame) -> Result<SeFrameEventSlices<'_>, ()> {
    // SAFETY: Forwarding the caller's guarantee that every non-null event
    // array holds at least its declared count of elements.
    unsafe {
        Ok(SeFrameEventSlices {
            touches: checked_slice(frame.touches, frame.touch_count)?,
            dodge_refreshes: checked_slice(frame.dodge_refreshes, frame.dodge_refresh_count)?,
            boost_pad_events: checked_slice(frame.boost_pad_events, frame.boost_pad_event_count)?,
            goals: checked_slice(frame.goals, frame.goal_count)?,
            player_stat_events: checked_slice(
                frame.player_stat_events,
                frame.player_stat_event_count,
            )?,
            demolishes: checked_slice(frame.demolishes, frame.demolish_count)?,
        })
    }
}

pub(crate) fn live_explicit_events(events: &SeFrameEventSlices<'_>) -> LiveExplicitEvents {
    LiveExplicitEvents {
        touches: events.touches.iter().map(live_touch_event).collect(),
        dodge_refreshes: events
            .dodge_refreshes
            .iter()
            .map(live_dodge_refreshed_event)
            .collect(),
        boost_pad_events: events
            .boost_pad_events
            .iter()
            .map(live_boost_pad_event)
            .collect(),
        goals: events.goals.iter().map(live_goal_event).collect(),
        player_stat_events: events
            .player_stat_events
            .iter()
            .map(live_player_stat_event)
            .collect(),
        demolishes: events.demolishes.iter().map(live_demolish_event).collect(),
    }
}

pub(crate) fn live_frame_data(frame: &SeFrame) -> LiveFrame {
    LiveFrame {
        frame_number: frame.frame_number,
        time: finite_f32(frame.time),
        dt: finite_f32(frame.dt),
        seconds_remaining: (frame.has_seconds_remaining != 0).then_some(frame.seconds_remaining),
        game_state: (frame.has_game_state != 0).then_some(frame.game_state),
        kickoff_countdown_time: (frame.has_kickoff_countdown_time != 0)
            .then_some(frame.kickoff_countdown_time),
        ball_has_been_hit: (frame.has_ball_has_been_hit != 0)
            .then_some(frame.ball_has_been_hit != 0),
        team_zero_score: (frame.has_team_zero_score != 0).then_some(frame.team_zero_score),
        team_one_score: (frame.has_team_one_score != 0).then_some(frame.team_one_score),
        possession_team_is_team_0: (frame.has_possession_team != 0)
            .then_some(frame.possession_team_is_team_0 != 0),
        scored_on_team_is_team_0: (frame.has_scored_on_team != 0)
            .then_some(frame.scored_on_team_is_team_0 != 0),
        live_play: (frame.has_live_play != 0).then_some(frame.live_play != 0),
        ball: (frame.has_ball != 0).then(|| rigid_body(frame.ball)),
        players: Vec::new(),
        events: LiveExplicitEvents::default(),
    }
}

/// Full [`SeFrame`] -> [`LiveFrame`] conversion. Fails (`Err`) only on
/// malformed slices: a null array pointer paired with a nonzero count.
pub(crate) unsafe fn live_frame_from_abi(frame: &SeFrame) -> Result<LiveFrame, ()> {
    // SAFETY: Forwarding the caller's pointer validity guarantees.
    let players = unsafe { checked_slice(frame.players, frame.player_count) }?;
    // SAFETY: Forwarding the caller's pointer validity guarantees.
    let events = unsafe { frame_event_slices(frame) }?;
    Ok(LiveFrame {
        // SAFETY: Forwarding the caller's pointer validity guarantees.
        players: unsafe { live_player_frames(players) },
        events: live_explicit_events(&events),
        ..live_frame_data(frame)
    })
}

pub(crate) unsafe fn live_match_context(context: &SeMatchContext) -> LiveMatchContext {
    LiveMatchContext {
        // SAFETY: Forwarding the caller's C-string validity guarantees.
        match_guid: unsafe { raw_c_string(context.match_guid) },
        playlist_id: (context.has_playlist_id != 0).then_some(context.playlist_id),
        // SAFETY: Forwarding the caller's C-string validity guarantees.
        map_name: unsafe { raw_c_string(context.map_name) },
    }
}
