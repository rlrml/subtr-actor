//! repr(C) structs shared with the C++ BakkesMod state-export plugin.
//!
//! The `Se` structs mirror the `Sa` shapes from
//! `bakkesmod/subtr-actor/rust/src/abi.rs` (so the C++ sampling code ports
//! mechanically) plus the superset fields carried by
//! [`subtr_actor_live::LivePlayerFrame`]: controller input, camera state,
//! dodge impulse/torque, and platform identity. Layouts are locked by
//! `src/lib_tests/abi_layout.rs` and mirrored in `include/state_export.h`;
//! any change here must be reflected in both.

use super::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SeQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for SeQuat {
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
pub struct SeRigidBody {
    pub location: SeVec3,
    pub rotation: SeQuat,
    pub linear_velocity: SeVec3,
    pub angular_velocity: SeVec3,
    pub has_linear_velocity: u8,
    pub has_angular_velocity: u8,
    pub sleeping: u8,
}

/// Controller input axes in `-1..1` plus button states.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeControllerInput {
    pub throttle: f32,
    pub steer: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub dodge_forward: f32,
    pub dodge_strafe: f32,
    pub handbrake: u8,
    pub jump: u8,
    pub activate_boost: u8,
    pub holding_boost: u8,
}

/// Replay-style camera state; each part is independently optional.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeCameraState {
    pub pitch: u8,
    pub yaw: u8,
    pub has_pitch: u8,
    pub has_yaw: u8,
    pub ball_cam_active: u8,
    pub has_ball_cam: u8,
}

/// Platform discriminants for [`SeRemoteId::platform`].
///
/// The conversion to [`boxcars::RemoteId`] maps every platform that is
/// lossless from `(platform, online_id | epic_id)`:
///
/// | value | platform    | boxcars mapping                                  |
/// |-------|-------------|--------------------------------------------------|
/// | 0     | none        | `None` (falls back to SplitScreen(player_index)) |
/// | 1     | steam       | `RemoteId::Steam(online_id)`                     |
/// | 2     | epic        | `RemoteId::Epic(epic_id)` (null/empty => `None`) |
/// | 3     | xbox        | `RemoteId::Xbox(online_id)`                      |
/// | 4     | psynet      | `None` (PsyNetId carries opaque payload bytes)   |
/// | 5     | switch      | `RemoteId::Switch(online_id)`                    |
/// | 6     | splitscreen | `RemoteId::SplitScreen(splitscreen_index)`       |
/// | 7     | playstation | `None` (Ps4Id carries a name + payload bytes)    |
/// | 8     | qq          | `RemoteId::QQ(online_id)`                        |
///
/// Unknown values convert to `None`.
pub const SE_REMOTE_ID_PLATFORM_NONE: u8 = 0;
pub const SE_REMOTE_ID_PLATFORM_STEAM: u8 = 1;
pub const SE_REMOTE_ID_PLATFORM_EPIC: u8 = 2;
pub const SE_REMOTE_ID_PLATFORM_XBOX: u8 = 3;
pub const SE_REMOTE_ID_PLATFORM_PSYNET: u8 = 4;
pub const SE_REMOTE_ID_PLATFORM_SWITCH: u8 = 5;
pub const SE_REMOTE_ID_PLATFORM_SPLITSCREEN: u8 = 6;
pub const SE_REMOTE_ID_PLATFORM_PLAYSTATION: u8 = 7;
pub const SE_REMOTE_ID_PLATFORM_QQ: u8 = 8;

/// Platform-qualified player identity (see the `SE_REMOTE_ID_PLATFORM_*`
/// mapping table). `epic_id` is only read when `platform` is epic.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeRemoteId {
    pub online_id: u64,
    pub epic_id: *const c_char,
    pub splitscreen_index: u32,
    pub platform: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SePlayerFrame {
    pub player_index: u32,
    pub player_name: *const c_char,
    pub is_team_0: u8,
    pub has_rigid_body: u8,
    pub rigid_body: SeRigidBody,
    pub boost_amount: f32,
    pub last_boost_amount: f32,
    pub boost_active: u8,
    pub jump_active: u8,
    pub double_jump_active: u8,
    pub dodge_active: u8,
    pub powerslide_active: u8,
    pub car_body_id: i32,
    pub has_car_body_id: u8,
    pub has_match_stats: u8,
    pub match_goals: i32,
    pub match_assists: i32,
    pub match_saves: i32,
    pub match_shots: i32,
    pub match_score: i32,
    pub has_input: u8,
    pub input: SeControllerInput,
    pub camera: SeCameraState,
    pub has_dodge_impulse: u8,
    pub dodge_impulse: SeVec3,
    pub has_dodge_torque: u8,
    pub dodge_torque: SeVec3,
    pub remote_id: SeRemoteId,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeEventTiming {
    pub frame_number: u64,
    pub time: f32,
    pub seconds_remaining: i32,
    pub has_timing: u8,
    pub has_seconds_remaining: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeTouchEvent {
    pub timing: SeEventTiming,
    pub player_index: u32,
    pub has_player: u8,
    pub is_team_0: u8,
    pub closest_approach_distance: f32,
    pub has_closest_approach_distance: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeDodgeRefreshedEvent {
    pub timing: SeEventTiming,
    pub player_index: u32,
    pub is_team_0: u8,
    pub counter_value: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeBoostPadEventKind {
    PickedUp = 1,
    Available = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SeBoostPadEvent {
    pub timing: SeEventTiming,
    pub pad_id: u32,
    pub kind: SeBoostPadEventKind,
    pub sequence: u8,
    pub player_index: u32,
    pub has_player: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeGoalEvent {
    pub timing: SeEventTiming,
    pub scoring_team_is_team_0: u8,
    pub player_index: u32,
    pub has_player: u8,
    pub team_zero_score: i32,
    pub has_team_zero_score: u8,
    pub team_one_score: i32,
    pub has_team_one_score: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SePlayerStatEventKind {
    Shot = 1,
    Save = 2,
    Assist = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SePlayerStatEvent {
    pub timing: SeEventTiming,
    pub player_index: u32,
    pub is_team_0: u8,
    pub kind: SePlayerStatEventKind,
    pub has_shot_ball: u8,
    pub shot_ball: SeRigidBody,
    pub has_shot_player: u8,
    pub shot_player: SeRigidBody,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeDemolishEvent {
    pub timing: SeEventTiming,
    pub attacker_index: u32,
    pub victim_index: u32,
    pub attacker_velocity: SeVec3,
    pub victim_velocity: SeVec3,
    pub victim_location: SeVec3,
    pub active_duration_seconds: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeFrame {
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
    pub team_zero_score: i32,
    pub has_team_zero_score: u8,
    pub team_one_score: i32,
    pub has_team_one_score: u8,
    pub possession_team_is_team_0: u8,
    pub has_possession_team: u8,
    pub scored_on_team_is_team_0: u8,
    pub has_scored_on_team: u8,
    pub live_play: u8,
    pub has_live_play: u8,
    pub has_ball: u8,
    pub ball: SeRigidBody,
    pub players: *const SePlayerFrame,
    pub player_count: usize,
    pub touches: *const SeTouchEvent,
    pub touch_count: usize,
    pub dodge_refreshes: *const SeDodgeRefreshedEvent,
    pub dodge_refresh_count: usize,
    pub boost_pad_events: *const SeBoostPadEvent,
    pub boost_pad_event_count: usize,
    pub goals: *const SeGoalEvent,
    pub goal_count: usize,
    pub player_stat_events: *const SePlayerStatEvent,
    pub player_stat_event_count: usize,
    pub demolishes: *const SeDemolishEvent,
    pub demolish_count: usize,
}

/// Server configuration for `state_export_engine_create` /
/// `state_export_engine_restart`. Zero values select defaults (see the header
/// docs); `port == 0` binds an ephemeral port (read it back via
/// `state_export_status`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeConfig {
    pub server_name: *const c_char,
    pub max_queued_frames: u32,
    pub max_client_queue: u32,
    pub port: u16,
    pub bind_any_interface: u8,
}

pub const SE_STATE_STOPPED: i32 = 0;
pub const SE_STATE_LISTENING: i32 = 1;
pub const SE_STATE_ERROR: i32 = 2;

/// Cheap engine status (atomics only), safe to poll from the game thread
/// every tick.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeStatus {
    pub state: i32,
    pub client_count: u32,
    pub port: u16,
    pub frames_sent: u64,
    pub frames_dropped: u64,
}

/// Match-level context that cannot be derived from player frames. All
/// pointers are nullable; a null pointer / zero `has_playlist_id` clears the
/// corresponding field.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SeMatchContext {
    pub match_guid: *const c_char,
    pub map_name: *const c_char,
    pub playlist_id: i32,
    pub has_playlist_id: u8,
}
