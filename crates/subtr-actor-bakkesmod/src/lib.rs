use std::ptr;
use std::slice;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{
    BallFrameState, BallSample, FrameInfo, GameplayState, HalfFlipCalculator, PlayerFrameState,
    PlayerSample, SpeedFlipCalculator, WavedashCalculator,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for SaQuat {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaRigidBody {
    pub location: SaVec3,
    pub rotation: SaQuat,
    pub linear_velocity: SaVec3,
    pub angular_velocity: SaVec3,
    pub has_linear_velocity: u8,
    pub has_angular_velocity: u8,
    pub sleeping: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaPlayerFrame {
    pub player_index: u32,
    pub is_team_0: u8,
    pub has_rigid_body: u8,
    pub rigid_body: SaRigidBody,
    pub boost_amount: f32,
    pub last_boost_amount: f32,
    pub boost_active: u8,
    pub dodge_active: u8,
    pub powerslide_active: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaLiveFrame {
    pub frame_number: u64,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: i32,
    pub has_seconds_remaining: u8,
    pub game_state: i32,
    pub has_game_state: u8,
    pub kickoff_countdown_time: i32,
    pub has_kickoff_countdown_time: u8,
    pub ball_has_been_hit: u8,
    pub has_ball_has_been_hit: u8,
    pub live_play: u8,
    pub has_ball: u8,
    pub ball: SaRigidBody,
    pub players: *const SaPlayerFrame,
    pub player_count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaMechanicKind {
    SpeedFlip = 1,
    HalfFlip = 2,
    Wavedash = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaMechanicEvent {
    pub kind: SaMechanicKind,
    pub player_index: u32,
    pub is_team_0: u8,
    pub frame_number: u64,
    pub time: f32,
    pub confidence: f32,
}

#[derive(Debug, Default)]
pub struct SaEngine {
    speed_flip: SpeedFlipCalculator,
    half_flip: HalfFlipCalculator,
    wavedash: WavedashCalculator,
    last_speed_flip_count: usize,
    last_half_flip_count: usize,
    last_wavedash_count: usize,
    pending_events: Vec<SaMechanicEvent>,
}

fn vec3(value: SaVec3) -> Vector3f {
    Vector3f {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

fn quat(value: SaQuat) -> Quaternion {
    Quaternion {
        x: value.x,
        y: value.y,
        z: value.z,
        w: value.w,
    }
}

fn rigid_body(value: SaRigidBody) -> RigidBody {
    RigidBody {
        location: vec3(value.location),
        rotation: quat(value.rotation),
        sleeping: value.sleeping != 0,
        linear_velocity: (value.has_linear_velocity != 0).then_some(vec3(value.linear_velocity)),
        angular_velocity: (value.has_angular_velocity != 0).then_some(vec3(value.angular_velocity)),
    }
}

fn player_id(index: u32) -> RemoteId {
    RemoteId::SplitScreen(index)
}

fn player_index(id: &RemoteId) -> u32 {
    match id {
        RemoteId::SplitScreen(index) => *index,
        _ => 0,
    }
}

fn frame_info(frame: &SaLiveFrame) -> FrameInfo {
    FrameInfo {
        frame_number: frame.frame_number as usize,
        time: frame.time,
        dt: frame.dt,
        seconds_remaining: (frame.has_seconds_remaining != 0).then_some(frame.seconds_remaining),
    }
}

fn gameplay_state(frame: &SaLiveFrame, players: &[SaPlayerFrame]) -> GameplayState {
    let mut counts = [0, 0];
    for player in players {
        counts[usize::from(player.is_team_0 == 0)] += 1;
    }

    GameplayState {
        game_state: (frame.has_game_state != 0).then_some(frame.game_state),
        ball_has_been_hit: (frame.has_ball_has_been_hit != 0)
            .then_some(frame.ball_has_been_hit != 0),
        kickoff_countdown_time: (frame.has_kickoff_countdown_time != 0)
            .then_some(frame.kickoff_countdown_time),
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: counts,
    }
}

fn ball_state(frame: &SaLiveFrame) -> BallFrameState {
    if frame.has_ball == 0 {
        BallFrameState::Missing
    } else {
        BallFrameState::Present(BallSample {
            rigid_body: rigid_body(frame.ball),
        })
    }
}

fn player_state(players: &[SaPlayerFrame]) -> PlayerFrameState {
    PlayerFrameState {
        players: players
            .iter()
            .map(|player| PlayerSample {
                player_id: player_id(player.player_index),
                is_team_0: player.is_team_0 != 0,
                rigid_body: (player.has_rigid_body != 0).then_some(rigid_body(player.rigid_body)),
                boost_amount: Some(player.boost_amount),
                last_boost_amount: Some(player.last_boost_amount),
                boost_active: player.boost_active != 0,
                dodge_active: player.dodge_active != 0,
                powerslide_active: player.powerslide_active != 0,
                match_goals: None,
                match_assists: None,
                match_saves: None,
                match_shots: None,
                match_score: None,
            })
            .collect(),
    }
}

fn push_new_events(engine: &mut SaEngine) {
    for event in &engine.speed_flip.events()[engine.last_speed_flip_count..] {
        engine.pending_events.push(SaMechanicEvent {
            kind: SaMechanicKind::SpeedFlip,
            player_index: player_index(&event.player),
            is_team_0: event.is_team_0 as u8,
            frame_number: event.frame as u64,
            time: event.time,
            confidence: event.confidence,
        });
    }
    engine.last_speed_flip_count = engine.speed_flip.events().len();

    for event in &engine.half_flip.events()[engine.last_half_flip_count..] {
        engine.pending_events.push(SaMechanicEvent {
            kind: SaMechanicKind::HalfFlip,
            player_index: player_index(&event.player),
            is_team_0: event.is_team_0 as u8,
            frame_number: event.frame as u64,
            time: event.time,
            confidence: event.confidence,
        });
    }
    engine.last_half_flip_count = engine.half_flip.events().len();

    for event in &engine.wavedash.events()[engine.last_wavedash_count..] {
        engine.pending_events.push(SaMechanicEvent {
            kind: SaMechanicKind::Wavedash,
            player_index: player_index(&event.player),
            is_team_0: event.is_team_0 as u8,
            frame_number: event.frame as u64,
            time: event.time,
            confidence: event.confidence,
        });
    }
    engine.last_wavedash_count = engine.wavedash.events().len();
}

/// Creates an opaque live-analysis engine.
///
/// The caller owns the returned pointer and must free it with
/// `subtr_actor_bakkesmod_engine_destroy`.
#[no_mangle]
pub extern "C" fn subtr_actor_bakkesmod_engine_create() -> *mut SaEngine {
    Box::into_raw(Box::new(SaEngine::default()))
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
    let frame_info = frame_info(frame);
    let gameplay = gameplay_state(frame, players);
    let ball = ball_state(frame);
    let players_state = player_state(players);
    let live_play = frame.live_play != 0;
    if engine
        .speed_flip
        .update_parts(&frame_info, &gameplay, &ball, &players_state, live_play)
        .is_err()
    {
        return -2;
    }
    if engine
        .half_flip
        .update(&frame_info, &players_state, live_play)
        .is_err()
    {
        return -2;
    }
    if engine
        .wavedash
        .update(&frame_info, &players_state, live_play)
        .is_err()
    {
        return -2;
    }

    push_new_events(engine);
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
/// Copies and removes pending events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaMechanicEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_events(
    engine: *mut SaEngine,
    out_events: *mut SaMechanicEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_events.len());
    ptr::copy_nonoverlapping(engine.pending_events.as_ptr(), out_events, count);
    engine.pending_events.drain(..count);
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_null_players_when_count_is_zero() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: 0,
            has_seconds_remaining: 0,
            game_state: 0,
            has_game_state: 0,
            kickoff_countdown_time: 0,
            has_kickoff_countdown_time: 0,
            ball_has_been_hit: 0,
            has_ball_has_been_hit: 0,
            live_play: 1,
            has_ball: 0,
            ball: SaRigidBody::default(),
            players: ptr::null(),
            player_count: 0,
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, 0);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn rejects_null_players_when_count_is_nonzero() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: 0,
            has_seconds_remaining: 0,
            game_state: 0,
            has_game_state: 0,
            kickoff_countdown_time: 0,
            has_kickoff_countdown_time: 0,
            ball_has_been_hit: 0,
            has_ball_has_been_hit: 0,
            live_play: 1,
            has_ball: 0,
            ball: SaRigidBody::default(),
            players: ptr::null(),
            player_count: 1,
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, -1);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }
}
