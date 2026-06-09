use super::*;

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
    pub player_name: *const c_char,
    pub is_team_0: u8,
    pub has_rigid_body: u8,
    pub rigid_body: SaRigidBody,
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
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaEventTiming {
    pub frame_number: u64,
    pub time: f32,
    pub seconds_remaining: i32,
    pub has_timing: u8,
    pub has_seconds_remaining: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaTouchEvent {
    pub timing: SaEventTiming,
    pub player_index: u32,
    pub has_player: u8,
    pub is_team_0: u8,
    pub closest_approach_distance: f32,
    pub has_closest_approach_distance: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaDodgeRefreshedEvent {
    pub timing: SaEventTiming,
    pub player_index: u32,
    pub is_team_0: u8,
    pub counter_value: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaBoostPadEventKind {
    PickedUp = 1,
    Available = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaBoostPadEvent {
    pub timing: SaEventTiming,
    pub pad_id: u32,
    pub kind: SaBoostPadEventKind,
    pub sequence: u8,
    pub player_index: u32,
    pub has_player: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaGoalEvent {
    pub timing: SaEventTiming,
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
pub enum SaPlayerStatEventKind {
    Shot = 1,
    Save = 2,
    Assist = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaPlayerStatEvent {
    pub timing: SaEventTiming,
    pub player_index: u32,
    pub is_team_0: u8,
    pub kind: SaPlayerStatEventKind,
    pub has_shot_ball: u8,
    pub shot_ball: SaRigidBody,
    pub has_shot_player: u8,
    pub shot_player: SaRigidBody,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaDemolishEvent {
    pub timing: SaEventTiming,
    pub attacker_index: u32,
    pub victim_index: u32,
    pub attacker_velocity: SaVec3,
    pub victim_velocity: SaVec3,
    pub victim_location: SaVec3,
    pub active_duration_seconds: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
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
    pub ball: SaRigidBody,
    pub players: *const SaPlayerFrame,
    pub player_count: usize,
    pub touches: *const SaTouchEvent,
    pub touch_count: usize,
    pub dodge_refreshes: *const SaDodgeRefreshedEvent,
    pub dodge_refresh_count: usize,
    pub boost_pad_events: *const SaBoostPadEvent,
    pub boost_pad_event_count: usize,
    pub goals: *const SaGoalEvent,
    pub goal_count: usize,
    pub player_stat_events: *const SaPlayerStatEvent,
    pub player_stat_event_count: usize,
    pub demolishes: *const SaDemolishEvent,
    pub demolish_count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaReplayScore {
    pub team_zero_score: i32,
    pub has_team_zero_score: u8,
    pub team_one_score: i32,
    pub has_team_one_score: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaMechanicKind {
    SpeedFlip = 1,
    HalfFlip = 2,
    Wavedash = 3,
    BallCarry = 4,
    AirDribble = 5,
    CeilingShot = 6,
    WallAerial = 7,
    WallAerialShot = 8,
    Center = 9,
    FlipReset = 10,
    DoubleTap = 11,
    Flick = 12,
    MustyFlick = 13,
    OneTimer = 14,
    Pass = 15,
    HalfVolley = 16,
    Whiff = 17,
    Bump = 18,
    Backboard = 19,
    BoostPickup = 20,
    Demo = 21,
    FiftyFifty = 22,
    AerialGoal = 23,
    HighAerialGoal = 24,
    LongDistanceGoal = 25,
    OwnHalfGoal = 26,
    EmptyNetGoal = 27,
    CounterAttackGoal = 28,
    FlickGoal = 29,
    DoubleTapGoal = 30,
    OneTimerGoal = 31,
    AirDribbleGoal = 32,
    FlipResetGoal = 33,
    HalfVolleyGoal = 34,
    Goal = 35,
    Shot = 36,
    Save = 37,
    Assist = 38,
    Death = 39,
    PassingGoal = 40,
    BumpGoal = 41,
    DemoGoal = 42,
    SustainedPressureGoal = 43,
    KickoffGoal = 44,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaReplayPlayerInfo {
    pub player_index: u32,
    pub is_team_0: u8,
    pub name: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaTeamEventKind {
    Rush = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaTeamEvent {
    pub kind: SaTeamEventKind,
    pub is_team_0: u8,
    pub start_frame: u64,
    pub end_frame: u64,
    pub start_time: f32,
    pub end_time: f32,
    pub attackers: u32,
    pub defenders: u32,
    pub confidence: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaGoalBuildupKind {
    CounterAttack = 1,
    SustainedPressure = 2,
    Other = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaGoalContextEvent {
    pub frame_number: u64,
    pub time: f32,
    pub scoring_team_is_team_0: u8,
    pub has_scorer: u8,
    pub scorer_index: u32,
    pub has_scoring_team_most_back_player: u8,
    pub scoring_team_most_back_player_index: u32,
    pub has_defending_team_most_back_player: u8,
    pub defending_team_most_back_player_index: u32,
    pub has_ball_position: u8,
    pub ball_position: SaVec3,
    pub has_ball_air_time_before_goal: u8,
    pub ball_air_time_before_goal: f32,
    pub goal_buildup: SaGoalBuildupKind,
}
