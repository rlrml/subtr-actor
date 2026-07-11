use boxcars::{RemoteId, RigidBody, Vector3f};
use serde::{Deserialize, Serialize};
use subtr_actor::PlayerId;

/// One sampled live game frame, owned and serialization-friendly.
///
/// This is the shared input model for live subtr-actor integrations: samplers
/// (e.g. the BakkesMod plugin) convert their host representation into this
/// struct, and the generator/view in this crate drive the analysis graph from
/// it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveFrame {
    pub frame_number: u64,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub kickoff_countdown_time: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub live_play: Option<bool>,
    pub ball: Option<RigidBody>,
    pub players: Vec<LivePlayerFrame>,
    pub events: LiveExplicitEvents,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LivePlayerFrame {
    pub player_index: u32,
    pub name: Option<String>,
    /// Platform-qualified identity when the sampler provides one. `None`
    /// falls back to `RemoteId::SplitScreen(player_index)`.
    pub remote_id: Option<RemoteId>,
    pub is_team_0: bool,
    pub rigid_body: Option<RigidBody>,
    pub boost_amount: f32,
    pub last_boost_amount: f32,
    pub boost_active: u8,
    pub jump_active: u8,
    pub double_jump_active: u8,
    pub dodge_active: u8,
    pub powerslide_active: bool,
    pub input: Option<LiveControllerInput>,
    pub camera: Option<LiveCameraState>,
    pub dodge_impulse: Option<[f32; 3]>,
    pub dodge_torque: Option<[f32; 3]>,
    pub car_body_id: Option<u32>,
    pub match_stats: Option<LiveMatchStats>,
}

impl LivePlayerFrame {
    pub fn canonical_player_id(&self) -> PlayerId {
        self.remote_id
            .clone()
            .unwrap_or(RemoteId::SplitScreen(self.player_index))
    }
}

/// Controller input axes in `-1..1` plus button states, when sampled.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveControllerInput {
    pub throttle: f32,
    pub steer: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub dodge_forward: f32,
    pub dodge_strafe: f32,
    pub handbrake: bool,
    pub jump: bool,
    pub activate_boost: bool,
    pub holding_boost: bool,
}

/// Replay-style camera state; each part may be independently unavailable.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveCameraState {
    pub pitch: Option<u8>,
    pub yaw: Option<u8>,
    pub ball_cam_active: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveMatchStats {
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub score: i32,
}

/// Event timing replicated by the sampler; `frame_and_time` and
/// `seconds_remaining` fall back to the enclosing frame when absent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveEventTiming {
    pub frame_and_time: Option<(u64, f32)>,
    pub seconds_remaining: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveTouchEvent {
    pub timing: LiveEventTiming,
    pub player: Option<PlayerId>,
    pub is_team_0: bool,
    pub closest_approach_distance: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveDodgeRefreshedEvent {
    pub timing: LiveEventTiming,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveBoostPadEventKind {
    PickedUp,
    Available,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveBoostPadEvent {
    pub timing: LiveEventTiming,
    pub pad_id: String,
    pub kind: LiveBoostPadEventKind,
    pub sequence: u8,
    pub player: Option<PlayerId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveGoalEvent {
    pub timing: LiveEventTiming,
    pub scoring_team_is_team_0: bool,
    pub player: Option<PlayerId>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePlayerStatEventKind {
    Shot,
    Save,
    Assist,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePlayerStatEvent {
    pub timing: LiveEventTiming,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub kind: LivePlayerStatEventKind,
    pub shot_ball: Option<RigidBody>,
    pub shot_player: Option<RigidBody>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveDemolishEvent {
    pub timing: LiveEventTiming,
    pub attacker: PlayerId,
    pub victim: PlayerId,
    pub attacker_velocity: Vector3f,
    pub victim_velocity: Vector3f,
    pub victim_location: Vector3f,
    pub active_duration_seconds: f32,
}

/// Explicit events replicated by the sampler for one frame, already resolved
/// to canonical player ids.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveExplicitEvents {
    pub touches: Vec<LiveTouchEvent>,
    pub dodge_refreshes: Vec<LiveDodgeRefreshedEvent>,
    pub boost_pad_events: Vec<LiveBoostPadEvent>,
    pub goals: Vec<LiveGoalEvent>,
    pub player_stat_events: Vec<LivePlayerStatEvent>,
    pub demolishes: Vec<LiveDemolishEvent>,
}

pub fn player_id(index: u32) -> PlayerId {
    RemoteId::SplitScreen(index)
}

pub fn player_index(id: &PlayerId) -> u32 {
    match id {
        RemoteId::SplitScreen(index) => *index,
        _ => 0,
    }
}
