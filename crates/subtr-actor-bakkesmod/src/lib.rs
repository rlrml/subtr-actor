#![allow(clippy::result_large_err)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::io::{Read, Write};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine as _;
use boxcars::{ParserBuilder, Quaternion, RemoteId, RigidBody, Vector3f};
use flate2::{
    read::{DeflateDecoder, ZlibDecoder},
    write::DeflateEncoder,
    Compression,
};
use subtr_actor::{
    boost_amount_to_percent, builtin_analysis_node_json, builtin_stats_graph_snapshot_json,
    builtin_stats_module_config_json, builtin_stats_module_frame_json, builtin_stats_module_json,
    builtin_stats_module_names, default_stats_timeline_config,
    geometry::apply_velocities_to_rigid_body,
    stats::analysis_graph::{
        builtin_analysis_node_aliases, builtin_analysis_node_names, graph_with_all_analysis_nodes,
        AnalysisGraph, StatsTimelineEventsState, StatsTimelineFrameState,
    },
    BackboardBounceEvent, BallFrameState, BallSample, BoostPadEvent, BoostPadEventKind,
    BoostPickupComparisonEvent, BumpEvent, CorePlayerStatsEvent, DemoEventSample,
    DemolishAttribute, DemolishInfo, DodgeRefreshedEvent, FiftyFiftyEvent, FrameEventsState,
    FrameInfo, FrameInput, GameplayPhase, GameplayState, GoalBuildupKind, GoalContextEvent,
    GoalEvent, GoalTagEvent, GoalTagKind, LivePlayState, PlayerFrameState, PlayerId, PlayerInfo,
    PlayerSample, PlayerStatEvent, PlayerStatEventKind, ProcessorView, ReplayFrameInputBuilder,
    ReplayMeta, ReplayStatsFrame, ReplayStatsTimeline, ReplayStatsTimelineEvents, RushEvent,
    ShotEventMetadata, StatsEventTiming, StatsTimelineCollector, StatsTimelineEventCollector,
    StatsTimelineTagEvent, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult,
    TimelineEvent, TimelineEventKind, TouchEvent, TouchStateCalculator, WhiffEvent,
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

pub struct SaEngine {
    graph: AnalysisGraph,
    live_events: SaLiveEventGenerator,
    live_event_history: SaLiveEventHistory,
    live_replay_meta_initialized: bool,
    live_replay_meta: Option<ReplayMeta>,
    live_replay_meta_signature: Vec<(RemoteId, bool, Option<String>)>,
    emitted_mechanic_ids: HashSet<String>,
    emitted_team_event_ids: HashSet<String>,
    emitted_goal_context_ids: HashSet<String>,
    graph_info_json: Vec<u8>,
    timeline_frames: Vec<ReplayStatsFrame>,
    pending_events: Vec<SaMechanicEvent>,
    pending_team_events: Vec<SaTeamEvent>,
    pending_goal_context_events: Vec<SaGoalContextEvent>,
}

pub struct SaReplayAnnotations {
    events: Vec<SaMechanicEvent>,
    frames: Vec<ReplayStatsFrame>,
    players: Vec<SaReplayPlayerInfo>,
    _player_names: Vec<CString>,
    cursor: usize,
    last_poll_time: f32,
    initialized: bool,
}

fn replay_annotation_frame_at_time(
    annotations: &SaReplayAnnotations,
    replay_time: f32,
) -> Option<&ReplayStatsFrame> {
    annotations
        .frames
        .iter()
        .take_while(|frame| frame.time <= replay_time + f32::EPSILON)
        .last()
        .or_else(|| annotations.frames.first())
}

const LIVE_GRAPH_OUTPUT_NAMES: &[&str] = &[
    "events",
    "frame",
    "timeline",
    "stats",
    "analysis_nodes",
    "event_history",
    "graph_info",
];
const LIVE_EVENT_HISTORY_FIELD_NAMES: &[&str] = &[
    "active_demos",
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
];
const REQUIRED_EVENT_HISTORY_FIELD_NAMES: &[&str] = &[
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
];
const LIVE_GRAPH_EVENT_FIELD_NAMES: &[&str] = &[
    "timeline",
    "mechanics",
    "goal_context",
    "core_player",
    "core_team",
    "possession",
    "pressure",
    "territorial_pressure",
    "movement",
    "positioning",
    "rotation_player",
    "rotation_team",
    "backboard",
    "ball_carry",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "double_tap",
    "fifty_fifty",
    "flick",
    "musty_flick",
    "one_timer",
    "pass",
    "pass_last_completed",
    "goal_tags",
    "rush",
    "speed_flip",
    "half_flip",
    "half_volley",
    "wavedash",
    "whiff",
    "dodge_reset",
    "powerslide",
    "boost_pickups",
    "boost_ledger",
    "boost_state",
    "bump",
    "touch",
    "touch_last_touch",
    "touch_ball_movement",
];
const REQUIRED_GRAPH_EVENT_FIELD_NAMES: &[&str] = &["timeline", "goal_context", "boost_pickups"];

impl Default for SaEngine {
    fn default() -> Self {
        let mut graph = live_analysis_graph();
        let graph_info_json = serialize_graph_info(&mut graph);
        Self {
            graph,
            live_events: SaLiveEventGenerator::default(),
            live_event_history: SaLiveEventHistory::default(),
            live_replay_meta_initialized: false,
            live_replay_meta: None,
            live_replay_meta_signature: Vec::new(),
            emitted_mechanic_ids: HashSet::new(),
            emitted_team_event_ids: HashSet::new(),
            emitted_goal_context_ids: HashSet::new(),
            graph_info_json,
            timeline_frames: Vec::new(),
            pending_events: Vec::new(),
            pending_team_events: Vec::new(),
            pending_goal_context_events: Vec::new(),
        }
    }
}

fn live_analysis_graph() -> AnalysisGraph {
    graph_with_all_analysis_nodes()
}

fn serialize_graph_info(graph: &mut AnalysisGraph) -> Vec<u8> {
    let dag = graph.render_ascii_dag().unwrap_or_default();
    let node_names = graph.node_names().collect::<Vec<_>>();
    let callable_analysis_node_names = callable_analysis_node_names_for_graph(graph);
    serde_json::to_vec(&serde_json::json!({
        "builtin_analysis_node_names": builtin_analysis_node_names(),
        "builtin_analysis_node_aliases": builtin_analysis_node_aliases(),
        "callable_analysis_node_names": callable_analysis_node_names,
        "builtin_stats_module_names": builtin_stats_module_names(),
        "graph_output_names": LIVE_GRAPH_OUTPUT_NAMES,
        "graph_event_field_names": LIVE_GRAPH_EVENT_FIELD_NAMES,
        "required_graph_event_field_names": REQUIRED_GRAPH_EVENT_FIELD_NAMES,
        "event_history_field_names": LIVE_EVENT_HISTORY_FIELD_NAMES,
        "required_event_history_field_names": REQUIRED_EVENT_HISTORY_FIELD_NAMES,
        "node_names": node_names,
        "dag": dag,
    }))
    .unwrap_or_default()
}

#[derive(Clone, Default)]
struct SaLiveEventGenerator {
    touch_state: TouchStateCalculator,
    live_play_tracker: subtr_actor::LivePlayTracker,
    dodge_refresh_counters: Vec<(RemoteId, i32)>,
    active_demos: Vec<SaActiveDemo>,
    known_demolishes: Vec<(DemoEventSample, usize)>,
    boost_pad_pickup_sequence_times: HashMap<(String, u8), f32>,
    last_goal_event: Option<GoalEvent>,
}

#[derive(Clone, Default)]
struct SaLiveEventHistory {
    demo_events: Vec<DemolishInfo>,
    boost_pad_events: Vec<BoostPadEvent>,
    touch_events: Vec<TouchEvent>,
    dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    player_stat_events: Vec<PlayerStatEvent>,
    goal_events: Vec<GoalEvent>,
}

impl SaLiveEventHistory {
    fn append_frame_events(&mut self, events: &FrameEventsState) {
        self.demo_events.extend(events.demo_events.iter().cloned());
        self.boost_pad_events
            .extend(events.boost_pad_events.iter().cloned());
        self.touch_events
            .extend(events.touch_events.iter().cloned());
        self.dodge_refreshed_events
            .extend(events.dodge_refreshed_events.iter().cloned());
        self.player_stat_events
            .extend(events.player_stat_events.iter().cloned());
        self.goal_events.extend(events.goal_events.iter().cloned());
    }
}

#[derive(Debug, Clone)]
struct SaActiveDemo {
    sample: DemoEventSample,
    expires_at: f32,
}

fn vec3(value: SaVec3) -> Vector3f {
    Vector3f {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

fn zero_vec3() -> Vector3f {
    Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
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

fn player_frame_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<Vector3f> {
    players
        .player_position(player_id)
        .map(|[x, y, z]| Vector3f { x, y, z })
}

fn player_frame_position_array(
    players: &PlayerFrameState,
    player_id: &PlayerId,
) -> Option<[f32; 3]> {
    players.player_position(player_id)
}

fn live_car_actor_id(id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
    let Some(index) = SaLiveProcessorView::player_index(id) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    let Ok(index) = i32::try_from(index) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    Ok(boxcars::ActorId(index))
}

fn live_demolish_attribute(
    attacker: &PlayerId,
    victim: &PlayerId,
    demolish: Option<&DemolishInfo>,
) -> SubtrActorResult<DemolishAttribute> {
    Ok(DemolishAttribute::Fx(boxcars::DemolishFx {
        custom_demo_flag: false,
        custom_demo_id: 0,
        attacker_flag: true,
        attacker: live_car_actor_id(attacker)?,
        victim_flag: true,
        victim: live_car_actor_id(victim)?,
        attack_velocity: demolish
            .map(|demolish| demolish.attacker_velocity)
            .unwrap_or_else(zero_vec3),
        victim_velocity: demolish
            .map(|demolish| demolish.victim_velocity)
            .unwrap_or_else(zero_vec3),
    }))
}

struct SaFrameEventSlices<'a> {
    touches: &'a [SaTouchEvent],
    dodge_refreshes: &'a [SaDodgeRefreshedEvent],
    boost_pad_events: &'a [SaBoostPadEvent],
    goals: &'a [SaGoalEvent],
    player_stat_events: &'a [SaPlayerStatEvent],
    demolishes: &'a [SaDemolishEvent],
}

struct SaLiveProcessorView<'a> {
    replay_meta: Option<&'a ReplayMeta>,
    frame: &'a SaLiveFrame,
    players: &'a [SaPlayerFrame],
    player_ids: Vec<PlayerId>,
    events: FrameEventsState,
    event_history: &'a SaLiveEventHistory,
}

impl<'a> SaLiveProcessorView<'a> {
    fn new(
        replay_meta: Option<&'a ReplayMeta>,
        frame: &'a SaLiveFrame,
        players: &'a [SaPlayerFrame],
        events: FrameEventsState,
        event_history: &'a SaLiveEventHistory,
    ) -> Self {
        Self {
            replay_meta,
            frame,
            players,
            player_ids: players
                .iter()
                .map(|player| player_id(player.player_index))
                .collect(),
            events,
            event_history,
        }
    }

    fn missing<T>(property: &'static str) -> SubtrActorResult<T> {
        SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState { property })
    }

    fn player_index(player_id: &PlayerId) -> Option<u32> {
        match player_id {
            RemoteId::SplitScreen(index) => Some(*index),
            _ => None,
        }
    }

    fn player(&self, player_id: &PlayerId) -> SubtrActorResult<&SaPlayerFrame> {
        let Some(index) = Self::player_index(player_id) else {
            return Self::missing("live player");
        };
        self.players
            .iter()
            .find(|player| player.player_index == index)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "live player",
                })
            })
    }
}

unsafe fn checked_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    if items.is_null() && count != 0 {
        return Err(());
    }
    if count == 0 {
        Ok(&[])
    } else {
        Ok(slice::from_raw_parts(items, count))
    }
}

unsafe fn frame_event_slices(frame: &SaLiveFrame) -> Result<SaFrameEventSlices<'_>, ()> {
    Ok(SaFrameEventSlices {
        touches: checked_slice(frame.touches, frame.touch_count)?,
        dodge_refreshes: checked_slice(frame.dodge_refreshes, frame.dodge_refresh_count)?,
        boost_pad_events: checked_slice(frame.boost_pad_events, frame.boost_pad_event_count)?,
        goals: checked_slice(frame.goals, frame.goal_count)?,
        player_stat_events: checked_slice(frame.player_stat_events, frame.player_stat_event_count)?,
        demolishes: checked_slice(frame.demolishes, frame.demolish_count)?,
    })
}

impl ProcessorView for SaLiveProcessorView<'_> {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        self.replay_meta
            .cloned()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))
    }

    fn player_count(&self) -> usize {
        self.players.len()
    }

    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_> {
        Box::new(self.player_ids.iter())
    }

    fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        let mut counts = [0, 0];
        for player in self.players {
            counts[usize::from(player.is_team_0 == 0)] += 1;
        }
        counts
    }

    fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        (self.frame.has_seconds_remaining != 0)
            .then_some(self.frame.seconds_remaining)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "seconds_remaining",
                })
            })
    }

    fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        (self.frame.has_game_state != 0)
            .then_some(self.frame.game_state)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "game_state",
                })
            })
    }

    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        (self.frame.has_kickoff_countdown_time != 0)
            .then_some(self.frame.kickoff_countdown_time)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "kickoff_countdown_time",
                })
            })
    }

    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        (self.frame.has_ball_has_been_hit != 0)
            .then_some(self.frame.ball_has_been_hit != 0)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "ball_has_been_hit",
                })
            })
    }

    fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        Ok(false)
    }

    fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        if self.frame.has_team_zero_score != 0 && self.frame.has_team_one_score != 0 {
            Ok((self.frame.team_zero_score, self.frame.team_one_score))
        } else {
            Self::missing("team_scores")
        }
    }

    fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        (self.frame.has_possession_team != 0)
            .then_some(if self.frame.possession_team_is_team_0 != 0 {
                0
            } else {
                1
            })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "possession_team",
                })
            })
    }

    fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        (self.frame.has_scored_on_team != 0)
            .then_some(if self.frame.scored_on_team_is_team_0 != 0 {
                0
            } else {
                1
            })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "scored_on_team",
                })
            })
    }

    fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<RigidBody> {
        (self.frame.has_ball != 0)
            .then(|| rigid_body(self.frame.ball))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "ball",
                })
            })
    }

    fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_interpolated_ball_rigid_body(
        &self,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<RigidBody> {
        let player = self.player(player_id)?;
        (player.has_rigid_body != 0)
            .then(|| rigid_body(player.rigid_body))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "player rigid body",
                })
            })
    }

    fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let player = self.player(player_id)?;
        player_name(player).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "player name",
            })
        })
    }

    fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        Ok(if self.get_player_is_team_0(player_id)? {
            "0".to_owned()
        } else {
            "1".to_owned()
        })
    }

    fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.is_team_0 != 0)
    }

    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId> {
        let Some(index) = u32::try_from(actor_id.0).ok() else {
            return Err(SubtrActorError::new(
                SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                },
            ));
        };
        self.players
            .iter()
            .find(|player| player.player_index == index)
            .map(|player| player_id(player.player_index))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                })
            })
    }

    fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.boost_amount)
    }

    fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.last_boost_amount)
    }

    fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_player_boost_level(player_id)
            .map(boost_amount_to_percent)
    }

    fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.boost_active)
    }

    fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.jump_active)
    }

    fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.double_jump_active)
    }

    fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.dodge_active)
    }

    fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.powerslide_active != 0)
    }

    fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_assists)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match assists",
                })
            })
    }

    fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_goals)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match goals",
                })
            })
    }

    fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_saves)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match saves",
                })
            })
    }

    fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_score)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match score",
                })
            })
    }

    fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_shots)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match shots",
                })
            })
    }

    fn get_active_demos(&self) -> SubtrActorResult<Vec<DemolishAttribute>> {
        let mut seen = HashSet::new();
        let mut demos = Vec::new();
        for sample in &self.events.active_demos {
            if !seen.insert((sample.attacker.clone(), sample.victim.clone())) {
                continue;
            }
            let demolish = self.events.demo_events.iter().find(|demolish| {
                demolish.attacker == sample.attacker && demolish.victim == sample.victim
            });
            demos.push(live_demolish_attribute(
                &sample.attacker,
                &sample.victim,
                demolish,
            )?);
        }
        Ok(demos)
    }

    fn demolishes(&self) -> &[DemolishInfo] {
        &self.event_history.demo_events
    }

    fn boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.event_history.boost_pad_events
    }

    fn touch_events(&self) -> &[TouchEvent] {
        &self.event_history.touch_events
    }

    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.event_history.dodge_refreshed_events
    }

    fn player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.event_history.player_stat_events
    }

    fn goal_events(&self) -> &[GoalEvent] {
        &self.event_history.goal_events
    }

    fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
        &self.events.active_demos
    }

    fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
        &self.events.demo_events
    }

    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.events.boost_pad_events
    }

    fn current_frame_touch_events(&self) -> &[TouchEvent] {
        &self.events.touch_events
    }

    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.events.dodge_refreshed_events
    }

    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.events.player_stat_events
    }

    fn current_frame_goal_events(&self) -> &[GoalEvent] {
        &self.events.goal_events
    }
}

fn find_counter(counters: &[(RemoteId, i32)], player_id: &RemoteId) -> Option<i32> {
    counters
        .iter()
        .find_map(|(id, value)| (id == player_id).then_some(*value))
}

fn set_counter(counters: &mut Vec<(RemoteId, i32)>, player_id: RemoteId, value: i32) {
    if let Some((_, counter)) = counters.iter_mut().find(|(id, _)| id == &player_id) {
        *counter = value;
    } else {
        counters.push((player_id, value));
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
        team_zero_score: (frame.has_team_zero_score != 0).then_some(frame.team_zero_score),
        team_one_score: (frame.has_team_one_score != 0).then_some(frame.team_one_score),
        possession_team_is_team_0: (frame.has_possession_team != 0)
            .then_some(frame.possession_team_is_team_0 != 0),
        scored_on_team_is_team_0: (frame.has_scored_on_team != 0)
            .then_some(frame.scored_on_team_is_team_0 != 0),
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
                match_goals: (player.has_match_stats != 0).then_some(player.match_goals),
                match_assists: (player.has_match_stats != 0).then_some(player.match_assists),
                match_saves: (player.has_match_stats != 0).then_some(player.match_saves),
                match_shots: (player.has_match_stats != 0).then_some(player.match_shots),
                match_score: (player.has_match_stats != 0).then_some(player.match_score),
            })
            .collect(),
    }
}

fn explicit_live_play_state(frame: &SaLiveFrame) -> Option<LivePlayState> {
    if frame.has_live_play == 0 {
        return None;
    }

    let is_live_play = frame.live_play != 0;
    Some(LivePlayState {
        gameplay_phase: if is_live_play {
            GameplayPhase::ActivePlay
        } else {
            GameplayPhase::Unknown
        },
        is_live_play,
    })
}

fn event_frame_and_time(frame: &FrameInfo, timing: SaEventTiming) -> (usize, f32) {
    if timing.has_timing != 0 {
        (timing.frame_number as usize, timing.time)
    } else {
        (frame.frame_number, frame.time)
    }
}

fn event_seconds_remaining(frame: &FrameInfo, timing: SaEventTiming) -> i32 {
    if timing.has_seconds_remaining != 0 {
        timing.seconds_remaining
    } else {
        frame.seconds_remaining.unwrap_or_default()
    }
}

fn explicit_touch_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaTouchEvent],
) -> Vec<TouchEvent> {
    let mut accepted = Vec::new();
    let mut seen = HashSet::new();
    for event in events {
        let (frame_number, time) = event_frame_and_time(frame, event.timing);
        let player = (event.has_player != 0).then_some(player_id(event.player_index));
        let team_is_team_0 = event.is_team_0 != 0;
        if !seen.insert((frame_number, player.clone(), team_is_team_0)) {
            continue;
        }
        accepted.push(TouchEvent {
            time,
            frame: frame_number,
            team_is_team_0,
            player_position: player
                .as_ref()
                .and_then(|player_id| player_frame_position(players, player_id)),
            player,
            closest_approach_distance: (event.has_closest_approach_distance != 0)
                .then_some(event.closest_approach_distance),
            dodge_contact: false,
        });
    }
    accepted
}

fn explicit_dodge_refresh_keys(
    frame: &FrameInfo,
    events: &[SaDodgeRefreshedEvent],
) -> HashSet<(RemoteId, usize)> {
    events
        .iter()
        .map(|event| {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            (player_id(event.player_index), frame_number)
        })
        .collect()
}

const MIN_BOOST_PAD_RESPAWN_SECONDS: f32 = 4.0;
const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;
const MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;

fn boost_pad_pickup_sequence_is_recent(
    sequence_times: &HashMap<(String, u8), f32>,
    pad_id: &str,
    sequence: u8,
    event_time: f32,
) -> bool {
    sequence_times
        .get(&(pad_id.to_owned(), sequence))
        .is_some_and(|last_time| {
            let elapsed = event_time - *last_time;
            (0.0..MIN_BOOST_PAD_RESPAWN_SECONDS).contains(&elapsed)
        })
}

fn demolish_is_known(
    known_demolishes: &[(DemoEventSample, usize)],
    sample: &DemoEventSample,
    frame_number: usize,
) -> bool {
    known_demolishes.iter().any(|(existing, existing_frame)| {
        existing.attacker == sample.attacker
            && existing.victim == sample.victim
            && frame_number.abs_diff(*existing_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
    })
}

fn goal_event_is_duplicate(previous: &GoalEvent, candidate: &GoalEvent) -> bool {
    match (
        candidate.team_zero_score,
        candidate.team_one_score,
        previous.team_zero_score,
        previous.team_one_score,
    ) {
        (Some(team_zero), Some(team_one), Some(prev_team_zero), Some(prev_team_one)) => {
            team_zero == prev_team_zero && team_one == prev_team_one
        }
        _ => {
            previous.scoring_team_is_team_0 == candidate.scoring_team_is_team_0
                && (candidate.time - previous.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
        }
    }
}

fn explicit_player_stat_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaPlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let player = player_id(event.player_index);
            let shot = shot_event_metadata(event);
            PlayerStatEvent {
                time,
                frame: frame_number,
                player_position: shot
                    .as_ref()
                    .and_then(|shot| shot.player_position)
                    .or_else(|| player_frame_position(players, &player)),
                player,
                is_team_0: event.is_team_0 != 0,
                kind: match event.kind {
                    SaPlayerStatEventKind::Shot => PlayerStatEventKind::Shot,
                    SaPlayerStatEventKind::Save => PlayerStatEventKind::Save,
                    SaPlayerStatEventKind::Assist => PlayerStatEventKind::Assist,
                },
                shot,
            }
        })
        .collect()
}

fn shot_event_metadata(event: &SaPlayerStatEvent) -> Option<ShotEventMetadata> {
    if event.kind != SaPlayerStatEventKind::Shot || event.has_shot_ball == 0 {
        return None;
    }

    let ball_body = rigid_body(event.shot_ball);
    let player_body = (event.has_shot_player != 0).then(|| rigid_body(event.shot_player));
    Some(ShotEventMetadata::from_rigid_bodies(
        event.is_team_0 != 0,
        &ball_body,
        player_body.as_ref(),
    ))
}

fn explicit_demolish_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaDemolishEvent],
) -> Vec<DemolishInfo> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let attacker = player_id(event.attacker_index);
            let victim = player_id(event.victim_index);
            DemolishInfo {
                time,
                seconds_remaining: event_seconds_remaining(frame, event.timing),
                frame: frame_number,
                attacker_location: player_frame_position(players, &attacker),
                attacker,
                victim,
                attacker_velocity: vec3(event.attacker_velocity),
                victim_velocity: vec3(event.victim_velocity),
                victim_location: vec3(event.victim_location),
            }
        })
        .collect()
}

impl SaLiveEventGenerator {
    fn explicit_dodge_refreshed_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[SaDodgeRefreshedEvent],
    ) -> Vec<DodgeRefreshedEvent> {
        let mut dodge_refreshed_events = Vec::new();
        for event in events {
            let player = player_id(event.player_index);
            if find_counter(&self.dodge_refresh_counters, &player)
                .is_some_and(|previous| event.counter_value <= previous)
            {
                continue;
            }
            set_counter(
                &mut self.dodge_refresh_counters,
                player.clone(),
                event.counter_value,
            );
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            dodge_refreshed_events.push(DodgeRefreshedEvent {
                time,
                frame: frame_number,
                player_position: player_frame_position_array(players, &player),
                player,
                is_team_0: event.is_team_0 != 0,
                counter_value: event.counter_value,
            });
        }
        dodge_refreshed_events
    }

    fn explicit_demolish_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<SaDemolishEvent> {
        let mut accepted_events = Vec::new();
        for event in events {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            self.known_demolishes.retain(|(_, known_frame)| {
                frame_number.abs_diff(*known_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            });
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            if demolish_is_known(&self.known_demolishes, &sample, frame_number) {
                continue;
            }
            self.known_demolishes.push((sample, frame_number));
            accepted_events.push(*event);
        }
        accepted_events
    }

    fn sync_active_demos(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<DemoEventSample> {
        self.active_demos
            .retain(|demo| demo.expires_at + f32::EPSILON >= frame.time);

        for event in events {
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            let active_duration_seconds = if event.active_duration_seconds.is_finite()
                && event.active_duration_seconds > 0.0
            {
                event.active_duration_seconds
            } else {
                0.0
            };
            let (_, event_time) = event_frame_and_time(frame, event.timing);
            let expires_at = event_time + active_duration_seconds;
            if expires_at + f32::EPSILON < frame.time {
                continue;
            }
            if let Some(active_demo) = self.active_demos.iter_mut().find(|active_demo| {
                active_demo.sample.attacker == sample.attacker
                    && active_demo.sample.victim == sample.victim
            }) {
                active_demo.expires_at = expires_at;
            } else {
                self.active_demos.push(SaActiveDemo { sample, expires_at });
            }
        }

        self.active_demos
            .iter()
            .map(|active_demo| active_demo.sample.clone())
            .collect()
    }

    fn explicit_boost_pad_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[SaBoostPadEvent],
    ) -> Vec<BoostPadEvent> {
        let mut boost_pad_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let pad_id = event.pad_id.to_string();
            let kind = match event.kind {
                SaBoostPadEventKind::PickedUp => {
                    if boost_pad_pickup_sequence_is_recent(
                        &self.boost_pad_pickup_sequence_times,
                        &pad_id,
                        event.sequence,
                        time,
                    ) {
                        continue;
                    }
                    self.boost_pad_pickup_sequence_times
                        .insert((pad_id.clone(), event.sequence), time);
                    BoostPadEventKind::PickedUp {
                        sequence: event.sequence,
                    }
                }
                SaBoostPadEventKind::Available => BoostPadEventKind::Available,
            };
            let player = (event.has_player != 0).then_some(player_id(event.player_index));
            boost_pad_events.push(BoostPadEvent {
                time,
                frame: frame_number,
                pad_id,
                player_position: player
                    .as_ref()
                    .and_then(|player_id| player_frame_position(players, player_id)),
                player,
                kind,
            });
        }
        boost_pad_events
    }

    fn explicit_goal_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[SaGoalEvent],
    ) -> Vec<GoalEvent> {
        let mut goal_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let player = (event.has_player != 0).then_some(player_id(event.player_index));
            let goal_event = GoalEvent {
                time,
                frame: frame_number,
                scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
                player_position: player
                    .as_ref()
                    .and_then(|player_id| player_frame_position(players, player_id)),
                player,
                team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
                team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
            };
            if self
                .last_goal_event
                .as_ref()
                .is_some_and(|previous| goal_event_is_duplicate(previous, &goal_event))
            {
                continue;
            }
            self.last_goal_event = Some(goal_event.clone());
            goal_events.push(goal_event);
        }
        goal_events
    }

    fn frame_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        gameplay: &GameplayState,
        explicit_live_play: Option<LivePlayState>,
        explicit_events: &SaFrameEventSlices<'_>,
    ) -> (FrameEventsState, LivePlayState) {
        let explicit_touch_events = explicit_touch_events(frame, players, explicit_events.touches);
        let has_explicit_touch_events = !explicit_touch_events.is_empty();
        let explicit_dodge_refresh_keys =
            explicit_dodge_refresh_keys(frame, explicit_events.dodge_refreshes);
        let has_explicit_dodge_refreshed_events = !explicit_dodge_refresh_keys.is_empty();
        let explicit_dodge_refreshed_events =
            self.explicit_dodge_refreshed_events(frame, players, explicit_events.dodge_refreshes);
        let explicit_demolishes = self.explicit_demolish_events(frame, explicit_events.demolishes);
        let demo_events = explicit_demolish_events(frame, players, &explicit_demolishes);
        let active_demos = self.sync_active_demos(frame, &explicit_demolishes);
        let boost_pad_events =
            self.explicit_boost_pad_events(frame, players, explicit_events.boost_pad_events);
        let player_stat_events =
            explicit_player_stat_events(frame, players, explicit_events.player_stat_events);
        let goal_events = self.explicit_goal_events(frame, players, explicit_events.goals);
        let base_events = FrameEventsState {
            active_demos,
            demo_events,
            boost_pad_events,
            player_stat_events,
            goal_events,
            ..FrameEventsState::default()
        };
        let live_play = explicit_live_play.unwrap_or_else(|| {
            let mut gameplay = gameplay.clone();
            if has_explicit_touch_events || has_explicit_dodge_refreshed_events {
                if gameplay.ball_has_been_hit == Some(false) {
                    gameplay.ball_has_been_hit = Some(true);
                }
                if gameplay.kickoff_countdown_time.is_some_and(|time| time > 0) {
                    gameplay.kickoff_countdown_time = Some(0);
                    gameplay.game_state = None;
                }
            }
            self.live_play_tracker.state_parts(&gameplay, &base_events)
        });
        let touch_tracker_events = FrameEventsState {
            touch_events: explicit_touch_events,
            dodge_refreshed_events: explicit_dodge_refreshed_events.clone(),
            ..FrameEventsState::default()
        };
        let touch_state =
            self.touch_state
                .update(frame, ball, players, &touch_tracker_events, &live_play);
        let mut touch_events = touch_state.touch_events;
        if touch_events.is_empty() && has_explicit_touch_events {
            touch_events = touch_tracker_events.touch_events.clone();
        }
        let mut dodge_refreshed_events = explicit_dodge_refreshed_events;
        dodge_refreshed_events.sort_by_key(|event| event.counter_value);

        (
            FrameEventsState {
                touch_events,
                dodge_refreshed_events,
                ..base_events
            },
            live_play,
        )
    }
}

fn frame_input_from_live_state(
    live_events: &mut SaLiveEventGenerator,
    live_event_history: &mut SaLiveEventHistory,
    replay_meta: Option<&ReplayMeta>,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    let frame_info = frame_info(frame);
    let ball = ball_state(frame);
    let players = player_state(sampled_players);
    let gameplay = gameplay_state(frame, sampled_players);
    let explicit_live_play = explicit_live_play_state(frame);
    let (frame_events, live_play) = live_events.frame_events(
        &frame_info,
        &ball,
        &players,
        &gameplay,
        explicit_live_play,
        explicit_events,
    );
    live_event_history.append_frame_events(&frame_events);
    let processor = SaLiveProcessorView::new(
        replay_meta,
        frame,
        sampled_players,
        frame_events,
        live_event_history,
    );
    FrameInput::timeline_with_live_play_state(
        &processor,
        frame.frame_number as usize,
        frame.time,
        frame.dt,
        live_play,
    )
}

#[cfg(test)]
fn frame_input(
    engine: &mut SaEngine,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    frame_input_from_live_state(
        &mut engine.live_events,
        &mut engine.live_event_history,
        engine.live_replay_meta.as_ref(),
        frame,
        sampled_players,
        explicit_events,
    )
}

fn player_name(player: &SaPlayerFrame) -> Option<String> {
    if player.player_name.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(player.player_name) }
        .to_string_lossy()
        .trim()
        .to_owned();
    (!name.is_empty()).then_some(name)
}

fn default_live_player_name(player_id: &RemoteId) -> String {
    match player_id {
        RemoteId::SplitScreen(index) => format!("Player {index}"),
        _ => format!("{player_id:?}"),
    }
}

fn live_replay_meta_signature(players: &[SaPlayerFrame]) -> Vec<(RemoteId, bool, Option<String>)> {
    players
        .iter()
        .map(|player| {
            (
                player_id(player.player_index),
                player.is_team_0 != 0,
                player_name(player),
            )
        })
        .collect()
}

fn live_replay_meta(players: &[SaPlayerFrame]) -> ReplayMeta {
    let mut team_zero = Vec::new();
    let mut team_one = Vec::new();
    for player in players {
        let player_id = player_id(player.player_index);
        let info = PlayerInfo {
            remote_id: player_id.clone(),
            stats: None,
            name: player_name(player).unwrap_or_else(|| default_live_player_name(&player_id)),
        };
        if player.is_team_0 != 0 {
            team_zero.push(info);
        } else {
            team_one.push(info);
        }
    }
    ReplayMeta {
        team_zero,
        team_one,
        all_headers: Vec::new(),
    }
}

fn sync_live_replay_meta(
    engine: &mut SaEngine,
    players: &[SaPlayerFrame],
) -> subtr_actor::SubtrActorResult<()> {
    let signature = live_replay_meta_signature(players);
    if engine.live_replay_meta_initialized && engine.live_replay_meta_signature == signature {
        return Ok(());
    }

    let replay_meta = live_replay_meta(players);
    engine.graph.on_replay_meta(&replay_meta)?;
    engine.live_replay_meta_initialized = true;
    engine.live_replay_meta = Some(replay_meta);
    engine.live_replay_meta_signature = signature;
    Ok(())
}

fn has_duplicate_player_indices(players: &[SaPlayerFrame]) -> bool {
    let mut seen = HashSet::new();
    players
        .iter()
        .any(|player| !seen.insert(player.player_index))
}

fn mechanic_kind(kind: &str) -> Option<SaMechanicKind> {
    match kind {
        "air_dribble" => Some(SaMechanicKind::AirDribble),
        "ball_carry" => Some(SaMechanicKind::BallCarry),
        "ceiling_shot" => Some(SaMechanicKind::CeilingShot),
        "center" => Some(SaMechanicKind::Center),
        "double_tap" => Some(SaMechanicKind::DoubleTap),
        "flick" => Some(SaMechanicKind::Flick),
        "flip_reset" => Some(SaMechanicKind::FlipReset),
        "half_flip" => Some(SaMechanicKind::HalfFlip),
        "half_volley" => Some(SaMechanicKind::HalfVolley),
        "musty_flick" => Some(SaMechanicKind::MustyFlick),
        "one_timer" => Some(SaMechanicKind::OneTimer),
        "pass" => Some(SaMechanicKind::Pass),
        "speed_flip" => Some(SaMechanicKind::SpeedFlip),
        "wall_aerial" => Some(SaMechanicKind::WallAerial),
        "wall_aerial_shot" => Some(SaMechanicKind::WallAerialShot),
        "wavedash" => Some(SaMechanicKind::Wavedash),
        _ => None,
    }
}

fn mechanic_start(event: &StatsTimelineTagEvent) -> (usize, f32) {
    match event.timing {
        StatsEventTiming::Moment { frame, time } => (frame, time),
        StatsEventTiming::Span {
            start_frame,
            start_time,
            ..
        } => (start_frame, start_time),
    }
}

struct PendingGraphEvent {
    id: String,
    kind: SaMechanicKind,
    player_id: RemoteId,
    is_team_0: bool,
    frame_number: usize,
    time: f32,
    confidence: f32,
}

fn push_pending_graph_event(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: PendingGraphEvent,
) {
    if !emitted_mechanic_ids.insert(event.id) {
        return;
    }
    pending_events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: player_index(&event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

fn push_pending_team_event(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    id: String,
    event: SaTeamEvent,
) {
    if !emitted_team_event_ids.insert(id) {
        return;
    }
    pending_team_events.push(event);
}

fn push_pending_goal_context_event(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    id: String,
    event: SaGoalContextEvent,
) {
    if !emitted_goal_context_ids.insert(id) {
        return;
    }
    pending_goal_context_events.push(event);
}

fn push_mechanic_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    mechanics: &[StatsTimelineTagEvent],
) {
    for event in mechanics {
        let Some(kind) = mechanic_kind(&event.kind) else {
            continue;
        };
        let (frame_number, time) = mechanic_start(event);
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: event.id.clone(),
                kind,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number,
                time,
                confidence: 1.0,
            },
        );
    }
}

fn push_whiff_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    whiffs: &[WhiffEvent],
) {
    for (index, event) in whiffs.iter().enumerate() {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "whiff:{}:{}:{index}",
                    event.frame,
                    player_index(&event.player)
                ),
                kind: SaMechanicKind::Whiff,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn push_bump_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    bumps: &[BumpEvent],
) {
    for (index, event) in bumps.iter().enumerate() {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "bump:{}:{}:{}:{index}",
                    event.frame,
                    player_index(&event.initiator),
                    player_index(&event.victim)
                ),
                kind: SaMechanicKind::Bump,
                player_id: event.initiator.clone(),
                is_team_0: event.initiator_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }
}

fn push_backboard_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    backboard: &[BackboardBounceEvent],
) {
    for (index, event) in backboard.iter().enumerate() {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "backboard:{}:{}:{index}",
                    event.frame,
                    player_index(&event.player)
                ),
                kind: SaMechanicKind::Backboard,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn push_boost_pickup_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    boost_pickups: &[BoostPickupComparisonEvent],
) {
    for (index, event) in boost_pickups.iter().enumerate() {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "boost_pickup:{}:{}:{:?}:{:?}:{index}",
                    event.frame,
                    player_index(&event.player_id),
                    event.reported_frame,
                    event.inferred_frame
                ),
                kind: SaMechanicKind::BoostPickup,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn timeline_event_kind(kind: TimelineEventKind) -> SaMechanicKind {
    match kind {
        TimelineEventKind::Goal => SaMechanicKind::Goal,
        TimelineEventKind::Shot => SaMechanicKind::Shot,
        TimelineEventKind::Save => SaMechanicKind::Save,
        TimelineEventKind::Assist => SaMechanicKind::Assist,
        TimelineEventKind::Kill => SaMechanicKind::Demo,
        TimelineEventKind::Death => SaMechanicKind::Death,
    }
}

fn push_timeline_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    timeline: &[TimelineEvent],
) {
    let mut occurrence_by_key = HashMap::new();
    for event in timeline {
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        let frame_number = event.frame.unwrap_or(0);
        let event_key = format!(
            "{:?}:{}:{}:{}:{}",
            event.kind,
            event.time.to_bits(),
            frame_number,
            player_index(player_id),
            is_team_0 as u8
        );
        let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
        let id = format!("timeline:{event_key}:{occurrence}");
        *occurrence += 1;
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id,
                kind: timeline_event_kind(event.kind),
                player_id: player_id.clone(),
                is_team_0,
                frame_number,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn push_repeated_core_player_stat_events(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: &CorePlayerStatsEvent,
    kind: SaMechanicKind,
    count: i32,
) {
    for index in 0..count.max(0) {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "core_player:{:?}:{}:{}:{}",
                    kind,
                    event.frame,
                    player_index(&event.player),
                    index
                ),
                kind,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn push_core_player_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    core_player: &[CorePlayerStatsEvent],
) {
    for event in core_player {
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Shot,
            event.delta.shots,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Save,
            event.delta.saves,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Assist,
            event.delta.assists,
        );
    }
}

fn push_fifty_fifty_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    fifty_fifty: &[FiftyFiftyEvent],
) {
    for (index, event) in fifty_fifty.iter().enumerate() {
        let Some(winning_team_is_team_0) = event.winning_team_is_team_0 else {
            continue;
        };
        let Some(player_id) = (if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }) else {
            continue;
        };

        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "fifty_fifty:{}:{}:{}:{index}",
                    event.start_frame,
                    event.resolve_frame,
                    player_index(player_id)
                ),
                kind: SaMechanicKind::FiftyFifty,
                player_id: player_id.clone(),
                is_team_0: winning_team_is_team_0,
                frame_number: event.resolve_frame,
                time: event.resolve_time,
                confidence: 1.0,
            },
        );
    }
}

fn goal_tag_kind(kind: GoalTagKind) -> SaMechanicKind {
    match kind {
        GoalTagKind::AerialGoal => SaMechanicKind::AerialGoal,
        GoalTagKind::HighAerialGoal => SaMechanicKind::HighAerialGoal,
        GoalTagKind::LongDistanceGoal => SaMechanicKind::LongDistanceGoal,
        GoalTagKind::OwnHalfGoal => SaMechanicKind::OwnHalfGoal,
        GoalTagKind::EmptyNetGoal => SaMechanicKind::EmptyNetGoal,
        GoalTagKind::CounterAttackGoal => SaMechanicKind::CounterAttackGoal,
        GoalTagKind::FlickGoal => SaMechanicKind::FlickGoal,
        GoalTagKind::DoubleTapGoal => SaMechanicKind::DoubleTapGoal,
        GoalTagKind::OneTimerGoal => SaMechanicKind::OneTimerGoal,
        GoalTagKind::PassingGoal => SaMechanicKind::PassingGoal,
        GoalTagKind::AirDribbleGoal => SaMechanicKind::AirDribbleGoal,
        GoalTagKind::FlipResetGoal => SaMechanicKind::FlipResetGoal,
        GoalTagKind::HalfVolleyGoal => SaMechanicKind::HalfVolleyGoal,
    }
}

fn push_goal_tag_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    goal_tags: &[GoalTagEvent],
) {
    for event in goal_tags {
        let Some(scorer) = event.scorer.as_ref() else {
            continue;
        };
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "goal_tag:{}:{}:{:?}:{}",
                    event.goal_index,
                    event.frame,
                    event.kind,
                    player_index(scorer)
                ),
                kind: goal_tag_kind(event.kind),
                player_id: scorer.clone(),
                is_team_0: event.scoring_team_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }
}

fn push_rush_events_from_timeline(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    rush: &[RushEvent],
) {
    for event in rush {
        push_pending_team_event(
            pending_team_events,
            emitted_team_event_ids,
            format!(
                "rush:{}:{}:{}",
                event.start_frame, event.end_frame, event.is_team_0
            ),
            SaTeamEvent {
                kind: SaTeamEventKind::Rush,
                is_team_0: event.is_team_0 as u8,
                start_frame: event.start_frame as u64,
                end_frame: event.end_frame as u64,
                start_time: event.start_time,
                end_time: event.end_time,
                attackers: event.attackers as u32,
                defenders: event.defenders as u32,
                confidence: 1.0,
            },
        );
    }
}

fn goal_buildup_kind(kind: GoalBuildupKind) -> SaGoalBuildupKind {
    match kind {
        GoalBuildupKind::CounterAttack => SaGoalBuildupKind::CounterAttack,
        GoalBuildupKind::SustainedPressure => SaGoalBuildupKind::SustainedPressure,
        GoalBuildupKind::Other => SaGoalBuildupKind::Other,
    }
}

fn goal_context_position(position: Option<subtr_actor::GoalContextPosition>) -> SaVec3 {
    position
        .map(|position| SaVec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        })
        .unwrap_or_default()
}

fn push_goal_context_events_from_timeline(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    goal_context: &[GoalContextEvent],
) {
    for (index, event) in goal_context.iter().enumerate() {
        let scorer = event.scorer.as_ref();
        let scoring_team_most_back_player = event.scoring_team_most_back_player.as_ref();
        let defending_team_most_back_player = event.defending_team_most_back_player.as_ref();
        push_pending_goal_context_event(
            pending_goal_context_events,
            emitted_goal_context_ids,
            format!("goal_context:{}:{}:{index}", event.frame, event.time),
            SaGoalContextEvent {
                frame_number: event.frame as u64,
                time: event.time,
                scoring_team_is_team_0: event.scoring_team_is_team_0 as u8,
                has_scorer: scorer.is_some() as u8,
                scorer_index: scorer.map(player_index).unwrap_or(0),
                has_scoring_team_most_back_player: scoring_team_most_back_player.is_some() as u8,
                scoring_team_most_back_player_index: scoring_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_defending_team_most_back_player: defending_team_most_back_player.is_some()
                    as u8,
                defending_team_most_back_player_index: defending_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_ball_position: event.ball_position.is_some() as u8,
                ball_position: goal_context_position(event.ball_position),
                has_ball_air_time_before_goal: event.ball_air_time_before_goal.is_some() as u8,
                ball_air_time_before_goal: event.ball_air_time_before_goal.unwrap_or(0.0),
                goal_buildup: goal_buildup_kind(event.goal_buildup),
            },
        );
    }
}

fn replay_player_index_map(replay_meta: &ReplayMeta) -> HashMap<RemoteId, u32> {
    replay_meta
        .player_order()
        .enumerate()
        .map(|(index, player)| (player.remote_id.clone(), index as u32))
        .collect()
}

fn replay_annotation_players(replay_meta: &ReplayMeta) -> (Vec<CString>, Vec<SaReplayPlayerInfo>) {
    let mut names = Vec::new();
    let mut players = Vec::new();
    for (player_index, player) in replay_meta.player_order().enumerate() {
        names.push(CString::new(player.name.as_str()).unwrap_or_else(|_| {
            CString::new(player.name.replace('\0', "")).expect("nul bytes removed")
        }));
        players.push(SaReplayPlayerInfo {
            player_index: player_index as u32,
            is_team_0: (player_index < replay_meta.team_zero.len()) as u8,
            name: names.last().expect("player name was just pushed").as_ptr(),
        });
    }
    (names, players)
}

fn replay_player_index(index_map: &HashMap<RemoteId, u32>, id: &RemoteId) -> u32 {
    index_map
        .get(id)
        .copied()
        .unwrap_or_else(|| player_index(id))
}

fn push_replay_annotation(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    event: PendingGraphEvent,
) {
    if !emitted_ids.insert(event.id) {
        return;
    }
    events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: replay_player_index(index_map, &event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

fn replay_annotations_from_timeline(
    replay_meta: &ReplayMeta,
    timeline: &ReplayStatsTimelineEvents,
) -> Vec<SaMechanicEvent> {
    let index_map = replay_player_index_map(replay_meta);
    let mut events = Vec::new();
    let mut emitted_ids = HashSet::new();

    for event in &timeline.mechanics {
        let Some(kind) = mechanic_kind(&event.kind) else {
            continue;
        };
        let (frame_number, time) = mechanic_start(event);
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: event.id.clone(),
                kind,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number,
                time,
                confidence: 1.0,
            },
        );
    }

    for (index, event) in timeline.backboard.iter().enumerate() {
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_backboard:{}:{}:{index}",
                    event.frame,
                    replay_player_index(&index_map, &event.player)
                ),
                kind: SaMechanicKind::Backboard,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }

    for (index, event) in timeline.whiff.iter().enumerate() {
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_whiff:{}:{}:{index}",
                    event.frame,
                    replay_player_index(&index_map, &event.player)
                ),
                kind: SaMechanicKind::Whiff,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }

    for (index, event) in timeline.boost_pickups.iter().enumerate() {
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_boost_pickup:{}:{}:{:?}:{:?}:{index}",
                    event.frame,
                    replay_player_index(&index_map, &event.player_id),
                    event.reported_frame,
                    event.inferred_frame
                ),
                kind: SaMechanicKind::BoostPickup,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }

    for (index, event) in timeline.bump.iter().enumerate() {
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_bump:{}:{}:{}:{index}",
                    event.frame,
                    replay_player_index(&index_map, &event.initiator),
                    replay_player_index(&index_map, &event.victim)
                ),
                kind: SaMechanicKind::Bump,
                player_id: event.initiator.clone(),
                is_team_0: event.initiator_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }

    let mut occurrence_by_key = HashMap::new();
    for event in &timeline.timeline {
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        let frame_number = event.frame.unwrap_or(0);
        let event_key = format!(
            "replay_timeline:{:?}:{}:{}:{}:{}",
            event.kind,
            event.time.to_bits(),
            frame_number,
            replay_player_index(&index_map, player_id),
            is_team_0 as u8
        );
        let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
        let id = format!("{event_key}:{occurrence}");
        *occurrence += 1;
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id,
                kind: timeline_event_kind(event.kind),
                player_id: player_id.clone(),
                is_team_0,
                frame_number,
                time: event.time,
                confidence: 1.0,
            },
        );
    }

    for event in &timeline.core_player {
        for (kind, count) in [
            (SaMechanicKind::Shot, event.delta.shots),
            (SaMechanicKind::Save, event.delta.saves),
            (SaMechanicKind::Assist, event.delta.assists),
        ] {
            for index in 0..count.max(0) {
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_core_player:{:?}:{}:{}:{}",
                            kind,
                            event.frame,
                            replay_player_index(&index_map, &event.player),
                            index
                        ),
                        kind,
                        player_id: event.player.clone(),
                        is_team_0: event.is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
        }
    }

    for (index, event) in timeline.fifty_fifty.iter().enumerate() {
        let Some(winning_team_is_team_0) = event.winning_team_is_team_0 else {
            continue;
        };
        let Some(player_id) = (if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }) else {
            continue;
        };
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_fifty_fifty:{}:{}:{}:{index}",
                    event.start_frame,
                    event.resolve_frame,
                    replay_player_index(&index_map, player_id)
                ),
                kind: SaMechanicKind::FiftyFifty,
                player_id: player_id.clone(),
                is_team_0: winning_team_is_team_0,
                frame_number: event.resolve_frame,
                time: event.resolve_time,
                confidence: 1.0,
            },
        );
    }

    for event in &timeline.goal_tags {
        let Some(scorer) = event.scorer.as_ref() else {
            continue;
        };
        push_replay_annotation(
            &mut events,
            &mut emitted_ids,
            &index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_goal_tag:{}:{}:{:?}:{}",
                    event.goal_index,
                    event.frame,
                    event.kind,
                    replay_player_index(&index_map, scorer)
                ),
                kind: goal_tag_kind(event.kind),
                player_id: scorer.clone(),
                is_team_0: event.scoring_team_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }

    events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
    events
}

fn push_drainable_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    events: &ReplayStatsTimelineEvents,
) {
    push_mechanic_events_from_timeline(pending_events, emitted_mechanic_ids, &events.mechanics);
    push_backboard_events_from_timeline(pending_events, emitted_mechanic_ids, &events.backboard);
    push_whiff_events_from_timeline(pending_events, emitted_mechanic_ids, &events.whiff);
    push_boost_pickup_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.boost_pickups,
    );
    push_bump_events_from_timeline(pending_events, emitted_mechanic_ids, &events.bump);
    push_core_player_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.core_player,
    );
    push_timeline_events_from_timeline(pending_events, emitted_mechanic_ids, &events.timeline);
    push_fifty_fifty_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.fifty_fifty,
    );
    push_goal_tag_events_from_timeline(pending_events, emitted_mechanic_ids, &events.goal_tags);
    push_rush_events_from_timeline(pending_team_events, emitted_team_event_ids, &events.rush);
    push_goal_context_events_from_timeline(
        pending_goal_context_events,
        emitted_goal_context_ids,
        &events.goal_context,
    );
    pending_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
    pending_team_events.sort_by(|left, right| {
        left.end_time
            .total_cmp(&right.end_time)
            .then_with(|| left.end_frame.cmp(&right.end_frame))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.is_team_0.cmp(&right.is_team_0))
    });
    pending_goal_context_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| {
                left.scoring_team_is_team_0
                    .cmp(&right.scoring_team_is_team_0)
            })
    });
}

fn current_timeline_frame(graph: &AnalysisGraph) -> Option<ReplayStatsFrame> {
    graph
        .state::<StatsTimelineFrameState>()
        .and_then(|state| state.frame.clone())
}

fn record_timeline_frame(frames: &mut Vec<ReplayStatsFrame>, frame: ReplayStatsFrame) {
    if let Some(last_frame) = frames.last_mut() {
        if last_frame.frame_number == frame.frame_number {
            *last_frame = frame;
            return;
        }
    }
    frames.push(frame);
}

fn serialize_live_timeline(
    replay_meta: Option<&ReplayMeta>,
    events: ReplayStatsTimelineEvents,
    frames: Vec<ReplayStatsFrame>,
) -> Vec<u8> {
    let Some(replay_meta) = replay_meta else {
        return Vec::new();
    };
    serde_json::to_vec(&ReplayStatsTimeline {
        config: default_stats_timeline_config(),
        replay_meta: replay_meta.clone(),
        events,
        frames,
    })
    .unwrap_or_default()
}

fn serialize_stats_graph_snapshot(engine: &SaEngine) -> Vec<u8> {
    match builtin_stats_graph_snapshot_json(&engine.graph, engine.live_replay_meta.as_ref()) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn serialize_analysis_nodes_snapshot(engine: &SaEngine) -> Vec<u8> {
    match callable_analysis_nodes_json(&engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn callable_analysis_nodes_json(graph: &AnalysisGraph) -> SubtrActorResult<serde_json::Value> {
    let mut values = serde_json::Map::new();
    for node_name in callable_analysis_node_names_for_graph(graph) {
        values.insert(
            node_name.clone(),
            builtin_analysis_node_json(&node_name, graph)?,
        );
    }
    Ok(serde_json::Value::Object(values))
}

unsafe fn serialize_named_analysis_node(
    engine: *const SaEngine,
    node_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(node_name) = c_string_arg(node_name) else {
        return Vec::new();
    };
    match builtin_analysis_node_json(&node_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn callable_analysis_node_names_for_graph(graph: &AnalysisGraph) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.extend(graph.node_names().map(str::to_owned));
    names.extend(
        builtin_analysis_node_names()
            .iter()
            .map(|name| (*name).to_owned()),
    );
    names.extend(
        builtin_analysis_node_aliases()
            .iter()
            .map(|alias| alias.alias.to_owned()),
    );
    names.into_iter().collect()
}

fn callable_analysis_node_names(engine: &SaEngine) -> Vec<String> {
    callable_analysis_node_names_for_graph(&engine.graph)
}

fn serialize_analysis_node_names(engine: *const SaEngine) -> Vec<u8> {
    let Some(engine) = (unsafe { engine.as_ref() }) else {
        return Vec::new();
    };
    serde_json::to_vec(&callable_analysis_node_names(engine)).unwrap_or_default()
}

unsafe fn c_string_arg(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok().map(str::to_owned)
}

unsafe fn serialize_named_stats_module(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    match builtin_stats_module_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

unsafe fn serialize_named_stats_module_frame(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    let Some(replay_meta) = engine.live_replay_meta.as_ref() else {
        return Vec::new();
    };
    match builtin_stats_module_frame_json(&module_name, &engine.graph, replay_meta) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

unsafe fn serialize_named_stats_module_config(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    match builtin_stats_module_config_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn serialize_live_event_history(engine: &SaEngine) -> Vec<u8> {
    let active_demos: Vec<_> = engine
        .live_events
        .active_demos
        .iter()
        .map(|active_demo| {
            serde_json::json!({
                "attacker": &active_demo.sample.attacker,
                "victim": &active_demo.sample.victim,
            })
        })
        .collect();
    serde_json::to_vec(&serde_json::json!({
        "active_demos": active_demos,
        "demo_events": &engine.live_event_history.demo_events,
        "boost_pad_events": &engine.live_event_history.boost_pad_events,
        "touch_events": &engine.live_event_history.touch_events,
        "dodge_refreshed_events": &engine.live_event_history.dodge_refreshed_events,
        "player_stat_events": &engine.live_event_history.player_stat_events,
        "goal_events": &engine.live_event_history.goal_events,
    }))
    .unwrap_or_default()
}

fn current_timeline_events(graph: &AnalysisGraph) -> Option<ReplayStatsTimelineEvents> {
    graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
}

fn serialize_live_graph_output(engine: &SaEngine, output_name: &str) -> Option<Vec<u8>> {
    match output_name {
        "events" => current_timeline_events(&engine.graph)
            .map(|events| serde_json::to_vec(&events).unwrap_or_default()),
        "frame" => current_timeline_frame(&engine.graph)
            .map(|frame| serde_json::to_vec(&frame).unwrap_or_default()),
        "timeline" => current_timeline_events(&engine.graph).map(|events| {
            serialize_live_timeline(
                engine.live_replay_meta.as_ref(),
                events,
                engine.timeline_frames.clone(),
            )
        }),
        "stats" => Some(serialize_stats_graph_snapshot(engine)),
        "analysis_nodes" => Some(serialize_analysis_nodes_snapshot(engine)),
        "event_history" => Some(serialize_live_event_history(engine)),
        "graph_info" => Some(engine.graph_info_json.clone()),
        _ => None,
    }
}

fn inflate_stats_player_config_bytes(compressed: &[u8]) -> Option<Vec<u8>> {
    let mut raw_decoder = DeflateDecoder::new(compressed);
    let mut json = Vec::new();
    if raw_decoder.read_to_end(&mut json).is_ok()
        && serde_json::from_slice::<serde_json::Value>(&json).is_ok()
    {
        return Some(json);
    }

    let mut zlib_decoder = ZlibDecoder::new(compressed);
    let mut json = Vec::new();
    if zlib_decoder.read_to_end(&mut json).is_ok()
        && serde_json::from_slice::<serde_json::Value>(&json).is_ok()
    {
        return Some(json);
    }

    None
}

fn decode_stats_player_config_json(value: &CStr) -> Option<Vec<u8>> {
    let value = value.to_str().ok()?.trim();
    if value.starts_with('{') {
        return Some(value.as_bytes().to_vec());
    }

    let compressed = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .ok()?;
    inflate_stats_player_config_bytes(&compressed)
}

fn encode_stats_player_config_json(value: &CStr) -> Option<Vec<u8>> {
    let value = value.to_str().ok()?.trim();
    serde_json::from_str::<serde_json::Value>(value).ok()?;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(value.as_bytes()).ok()?;
    let compressed = encoder.finish().ok()?;
    Some(URL_SAFE_NO_PAD.encode(compressed).into_bytes())
}

fn refresh_timeline_graph_state(engine: &mut SaEngine) {
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

fn serialize_replay_annotation_frame(
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
    if engine.graph.finish().is_err() {
        return -2;
    }
    refresh_timeline_graph_state(engine);
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

#[no_mangle]
/// Returns the UTF-8 byte length of the current serialized graph event bundle.
///
/// The JSON payload is a `ReplayStatsTimelineEvents` value produced by the live
/// analysis graph after the most recent successful frame.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_events_json_len(engine: *const SaEngine) -> usize {
    engine
        .as_ref()
        .and_then(|engine| serialize_live_graph_output(engine, "events"))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the current serialized graph event bundle into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_events_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_events_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let Some(bytes) = serialize_live_graph_output(engine, "events") else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of the current serialized graph frame snapshot.
///
/// The JSON payload is a `ReplayStatsFrame` value produced by the live analysis
/// graph after the most recent successful frame.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_frame_json_len(engine: *const SaEngine) -> usize {
    engine
        .as_ref()
        .and_then(|engine| serialize_live_graph_output(engine, "frame"))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the current serialized graph frame snapshot into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_frame_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_frame_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let Some(bytes) = serialize_live_graph_output(engine, "frame") else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of the current serialized live stats timeline.
///
/// The JSON payload is a `ReplayStatsTimeline` value produced by the live
/// analysis graph. It contains the graph config, live replay metadata, all
/// timeline event families, and every frame snapshot observed by this engine.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_timeline_json_len(engine: *const SaEngine) -> usize {
    engine
        .as_ref()
        .and_then(|engine| serialize_live_graph_output(engine, "timeline"))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the current serialized live stats timeline into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_timeline_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_timeline_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let Some(bytes) = serialize_live_graph_output(engine, "timeline") else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of the current serialized live stats snapshot.
///
/// The JSON payload exposes the same builtin stats module surface as
/// `StatsCollector`: selected module names, snapshot config, aggregate module
/// JSON, and the current module-keyed frame snapshot when replay metadata and
/// frame state are available.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_json_len(engine: *const SaEngine) -> usize {
    engine
        .as_ref()
        .and_then(|engine| serialize_live_graph_output(engine, "stats"))
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the current serialized live stats snapshot into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_json_len` first to size the destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let Some(bytes) = serialize_live_graph_output(engine, "stats") else {
        return 0;
    };
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of one named builtin stats module JSON payload.
///
/// `module_name` must be one of the UTF-8 names reported by
/// `builtin_stats_module_names` in graph info JSON.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    serialize_named_stats_module(engine, module_name).len()
}

#[no_mangle]
/// Writes one named builtin stats module JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = serialize_named_stats_module(engine, module_name);
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of one named builtin stats module frame JSON payload.
///
/// Known modules with no per-frame snapshot return JSON `null`; unknown modules
/// and invalid inputs return length `0`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_frame_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    serialize_named_stats_module_frame(engine, module_name).len()
}

#[no_mangle]
/// Writes one named builtin stats module frame JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_frame_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_frame_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = serialize_named_stats_module_frame(engine, module_name);
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of one named builtin stats module config JSON payload.
///
/// Known modules with no snapshot config return JSON `null`; unknown modules and
/// invalid inputs return length `0`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `module_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_stats_module_config_json_len(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> usize {
    serialize_named_stats_module_config(engine, module_name).len()
}

#[no_mangle]
/// Writes one named builtin stats module config JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_stats_module_config_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `module_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_stats_module_config_json(
    engine: *const SaEngine,
    module_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = serialize_named_stats_module_config(engine, module_name);
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of one named live graph output JSON payload.
///
/// `output_name` must be one of `events`, `frame`, `timeline`, `stats`,
/// `analysis_nodes`, `event_history`, or `graph_info`, which are also reported
/// by graph info JSON.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `output_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_graph_output_json_len(
    engine: *const SaEngine,
    output_name: *const c_char,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    let Some(output_name) = c_string_arg(output_name) else {
        return 0;
    };
    serialize_live_graph_output(engine, &output_name)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes one named live graph output JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_graph_output_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `output_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_graph_output_json(
    engine: *const SaEngine,
    output_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    let Some(output_name) = c_string_arg(output_name) else {
        return 0;
    };
    let Some(bytes) = serialize_live_graph_output(engine, &output_name) else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of one named live analysis-node JSON payload.
///
/// `node_name` must be one of the names reported by
/// `subtr_actor_bakkesmod_analysis_node_names_json_len`. Calculator nodes use
/// the same graph-backed payloads as stats modules; signal/state nodes use
/// structured snapshots of their current graph state.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`. `node_name` must be a valid
/// null-terminated UTF-8 string.
pub unsafe extern "C" fn subtr_actor_bakkesmod_analysis_node_json_len(
    engine: *const SaEngine,
    node_name: *const c_char,
) -> usize {
    serialize_named_analysis_node(engine, node_name).len()
}

#[no_mangle]
/// Writes one named live analysis-node JSON payload into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_analysis_node_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `node_name` must be a valid
/// null-terminated UTF-8 string. `out_bytes` must point to writable storage for
/// at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_analysis_node_json(
    engine: *const SaEngine,
    node_name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = serialize_named_analysis_node(engine, node_name);
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of the callable analysis-node name registry.
///
/// The payload is a JSON string array containing every supported name for
/// `subtr_actor_bakkesmod_analysis_node_json_len`.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_analysis_node_names_json_len(
    engine: *const SaEngine,
) -> usize {
    serialize_analysis_node_names(engine).len()
}

#[no_mangle]
/// Writes the callable analysis-node name registry into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_analysis_node_names_json_len` first to size the
/// destination buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_analysis_node_names_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let bytes = serialize_analysis_node_names(engine);
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let count = max_bytes.min(bytes.len());
    ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count);
    count
}

#[no_mangle]
/// Returns the UTF-8 byte length of the serialized live graph metadata.
///
/// The JSON payload includes the builtin analysis-node registry, the actual
/// node names configured in this engine, and an ASCII DAG rendering.
///
/// # Safety
///
/// `engine` must either be null or a valid pointer returned by
/// `subtr_actor_bakkesmod_engine_create`.
pub unsafe extern "C" fn subtr_actor_bakkesmod_graph_info_json_len(
    engine: *const SaEngine,
) -> usize {
    engine
        .as_ref()
        .map(|engine| engine.graph_info_json.len())
        .unwrap_or(0)
}

#[no_mangle]
/// Writes the serialized live graph metadata into caller-owned storage.
///
/// Returns the number of bytes written. Call
/// `subtr_actor_bakkesmod_graph_info_json_len` first to size the destination
/// buffer.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_bytes` must point to writable
/// storage for at least `max_bytes` bytes.
pub unsafe extern "C" fn subtr_actor_bakkesmod_write_graph_info_json(
    engine: *const SaEngine,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(engine) = engine.as_ref() else {
        return 0;
    };
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }

    let count = max_bytes.min(engine.graph_info_json.len());
    ptr::copy_nonoverlapping(engine.graph_info_json.as_ptr(), out_bytes, count);
    count
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

#[no_mangle]
/// Copies and removes pending team-owned events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaTeamEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_team_events(
    engine: *mut SaEngine,
    out_events: *mut SaTeamEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_team_events.len());
    ptr::copy_nonoverlapping(engine.pending_team_events.as_ptr(), out_events, count);
    engine.pending_team_events.drain(..count);
    count
}

#[no_mangle]
/// Copies and removes pending goal-context events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaGoalContextEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_goal_context_events(
    engine: *mut SaEngine,
    out_events: *mut SaGoalContextEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_goal_context_events.len());
    ptr::copy_nonoverlapping(
        engine.pending_goal_context_events.as_ptr(),
        out_events,
        count,
    );
    engine.pending_goal_context_events.drain(..count);
    count
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
