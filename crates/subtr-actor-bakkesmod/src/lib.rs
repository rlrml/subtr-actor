use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{
    builtin_stats_graph_snapshot_json, builtin_stats_module_names, default_stats_timeline_config,
    stats::analysis_graph::{
        builtin_analysis_node_aliases, builtin_analysis_node_names, graph_with_all_analysis_nodes,
        AnalysisGraph, StatsTimelineEventsState, StatsTimelineFrameState,
        STATS_TIMELINE_MECHANIC_KINDS,
    },
    BackboardBounceEvent, BallFrameState, BallSample, BoostPadEvent, BoostPadEventKind,
    BoostPickupComparisonEvent, BumpEvent, DemoEventSample, DemolishAttribute, DemolishInfo,
    DodgeRefreshedEvent, FiftyFiftyEvent, FrameEventsState, FrameInfo, FrameInput, GameplayPhase,
    GameplayState, GoalBuildupKind, GoalContextEvent, GoalEvent, GoalTagEvent, GoalTagKind,
    LivePlayState, MechanicEvent, MechanicTiming, PlayerFrameState, PlayerId, PlayerInfo,
    PlayerSample, PlayerStatEvent, PlayerStatEventKind, ProcessorView, ReplayMeta,
    ReplayStatsFrame, ReplayStatsTimeline, ReplayStatsTimelineEvents, RushEvent, ShotEventMetadata,
    SubtrActorError, SubtrActorErrorVariant, SubtrActorResult, TimelineEvent, TimelineEventKind,
    TouchEvent, TouchState, TouchStateCalculator, WhiffEvent, WhiffEventKind,
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
pub struct SaTouchEvent {
    pub player_index: u32,
    pub has_player: u8,
    pub is_team_0: u8,
    pub closest_approach_distance: f32,
    pub has_closest_approach_distance: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaDodgeRefreshedEvent {
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
    pub pad_id: u32,
    pub kind: SaBoostPadEventKind,
    pub sequence: u8,
    pub player_index: u32,
    pub has_player: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SaGoalEvent {
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
    live_replay_meta_initialized: bool,
    live_replay_meta: Option<ReplayMeta>,
    live_replay_meta_signature: Vec<(RemoteId, bool, Option<String>)>,
    emitted_mechanic_ids: HashSet<String>,
    emitted_team_event_ids: HashSet<String>,
    emitted_goal_context_ids: HashSet<String>,
    events_json: Vec<u8>,
    frame_json: Vec<u8>,
    timeline_json: Vec<u8>,
    stats_json: Vec<u8>,
    graph_info_json: Vec<u8>,
    timeline_frames: Vec<ReplayStatsFrame>,
    pending_events: Vec<SaMechanicEvent>,
    pending_team_events: Vec<SaTeamEvent>,
    pending_goal_context_events: Vec<SaGoalContextEvent>,
}

impl Default for SaEngine {
    fn default() -> Self {
        let mut graph = live_analysis_graph();
        let graph_info_json = serialize_graph_info(&mut graph);
        Self {
            graph,
            live_events: SaLiveEventGenerator::default(),
            live_replay_meta_initialized: false,
            live_replay_meta: None,
            live_replay_meta_signature: Vec::new(),
            emitted_mechanic_ids: HashSet::new(),
            emitted_team_event_ids: HashSet::new(),
            emitted_goal_context_ids: HashSet::new(),
            events_json: Vec::new(),
            frame_json: Vec::new(),
            timeline_json: Vec::new(),
            stats_json: Vec::new(),
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
    serde_json::to_vec(&serde_json::json!({
        "builtin_analysis_node_names": builtin_analysis_node_names(),
        "builtin_analysis_node_aliases": builtin_analysis_node_aliases(),
        "builtin_stats_module_names": builtin_stats_module_names(),
        "node_names": node_names,
        "dag": dag,
    }))
    .unwrap_or_default()
}

#[derive(Default)]
struct SaLiveEventGenerator {
    touch_state: TouchStateCalculator,
    live_play_tracker: subtr_actor::LivePlayTracker,
    dodge_refresh_counters: Vec<(RemoteId, i32)>,
    active_demos: Vec<SaActiveDemo>,
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
}

impl<'a> SaLiveProcessorView<'a> {
    fn new(
        replay_meta: Option<&'a ReplayMeta>,
        frame: &'a SaLiveFrame,
        players: &'a [SaPlayerFrame],
        events: FrameEventsState,
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
        _target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        self.get_normalized_ball_rigid_body()
    }

    fn get_interpolated_ball_rigid_body(
        &self,
        _target_time: f32,
        _close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        self.get_normalized_ball_rigid_body()
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
        _target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        self.get_normalized_player_rigid_body(player_id)
    }

    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        _target_time: f32,
        _close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        self.get_normalized_player_rigid_body(player_id)
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
        if demos.is_empty() {
            for demolish in &self.events.demo_events {
                if !seen.insert((demolish.attacker.clone(), demolish.victim.clone())) {
                    continue;
                }
                demos.push(live_demolish_attribute(
                    &demolish.attacker,
                    &demolish.victim,
                    Some(demolish),
                )?);
            }
        }
        Ok(demos)
    }

    fn demolishes(&self) -> &[DemolishInfo] {
        &self.events.demo_events
    }

    fn boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.events.boost_pad_events
    }

    fn touch_events(&self) -> &[TouchEvent] {
        &self.events.touch_events
    }

    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.events.dodge_refreshed_events
    }

    fn player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.events.player_stat_events
    }

    fn goal_events(&self) -> &[GoalEvent] {
        &self.events.goal_events
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

fn explicit_touch_events(frame: &FrameInfo, events: &[SaTouchEvent]) -> Vec<TouchEvent> {
    events
        .iter()
        .map(|event| TouchEvent {
            time: frame.time,
            frame: frame.frame_number,
            team_is_team_0: event.is_team_0 != 0,
            player: (event.has_player != 0).then_some(player_id(event.player_index)),
            closest_approach_distance: (event.has_closest_approach_distance != 0)
                .then_some(event.closest_approach_distance),
        })
        .collect()
}

fn explicit_dodge_refreshed_events(
    frame: &FrameInfo,
    events: &[SaDodgeRefreshedEvent],
) -> Vec<DodgeRefreshedEvent> {
    events
        .iter()
        .map(|event| DodgeRefreshedEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: player_id(event.player_index),
            is_team_0: event.is_team_0 != 0,
            counter_value: event.counter_value,
        })
        .collect()
}

fn explicit_boost_pad_events(frame: &FrameInfo, events: &[SaBoostPadEvent]) -> Vec<BoostPadEvent> {
    events
        .iter()
        .map(|event| BoostPadEvent {
            time: frame.time,
            frame: frame.frame_number,
            pad_id: event.pad_id.to_string(),
            player: (event.has_player != 0).then_some(player_id(event.player_index)),
            kind: match event.kind {
                SaBoostPadEventKind::PickedUp => BoostPadEventKind::PickedUp {
                    sequence: event.sequence,
                },
                SaBoostPadEventKind::Available => BoostPadEventKind::Available,
            },
        })
        .collect()
}

fn explicit_goal_events(frame: &FrameInfo, events: &[SaGoalEvent]) -> Vec<GoalEvent> {
    events
        .iter()
        .map(|event| GoalEvent {
            time: frame.time,
            frame: frame.frame_number,
            scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
            player: (event.has_player != 0).then_some(player_id(event.player_index)),
            team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
            team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
        })
        .collect()
}

fn explicit_player_stat_events(
    frame: &FrameInfo,
    events: &[SaPlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    events
        .iter()
        .map(|event| PlayerStatEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: player_id(event.player_index),
            is_team_0: event.is_team_0 != 0,
            kind: match event.kind {
                SaPlayerStatEventKind::Shot => PlayerStatEventKind::Shot,
                SaPlayerStatEventKind::Save => PlayerStatEventKind::Save,
                SaPlayerStatEventKind::Assist => PlayerStatEventKind::Assist,
            },
            shot: shot_event_metadata(event),
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

fn explicit_demolish_events(frame: &FrameInfo, events: &[SaDemolishEvent]) -> Vec<DemolishInfo> {
    events
        .iter()
        .map(|event| DemolishInfo {
            time: frame.time,
            seconds_remaining: frame.seconds_remaining.unwrap_or_default(),
            frame: frame.frame_number,
            attacker: player_id(event.attacker_index),
            victim: player_id(event.victim_index),
            attacker_velocity: vec3(event.attacker_velocity),
            victim_velocity: vec3(event.victim_velocity),
            victim_location: vec3(event.victim_location),
        })
        .collect()
}

fn infer_dodge_refreshed_events(
    frame: &FrameInfo,
    ball: &BallFrameState,
    players: &PlayerFrameState,
    touch_events: &[subtr_actor::TouchEvent],
    counters: &mut Vec<(RemoteId, i32)>,
) -> Vec<DodgeRefreshedEvent> {
    const MIN_PLAYER_HEIGHT: f32 = 95.0;
    const MIN_BALL_HEIGHT: f32 = 80.0;
    const MAX_CENTER_DISTANCE: f32 = 180.0;
    const MAX_LOCAL_VERTICAL_OFFSET: f32 = 140.0;

    let Some(ball) = ball.sample() else {
        return Vec::new();
    };
    let ball_position = subtr_actor::vec_to_glam(&ball.rigid_body.location);
    if ball_position.z < MIN_BALL_HEIGHT {
        return Vec::new();
    }

    let mut events = Vec::new();
    for touch in touch_events {
        let Some(player_id) = touch.player.as_ref() else {
            continue;
        };
        let Some(player) = players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
        else {
            continue;
        };
        let Some(player_rigid_body) = player.rigid_body.as_ref() else {
            continue;
        };

        let player_position = subtr_actor::vec_to_glam(&player_rigid_body.location);
        if player_position.z < MIN_PLAYER_HEIGHT {
            continue;
        }

        let relative_ball_position = ball_position - player_position;
        if !relative_ball_position.is_finite()
            || relative_ball_position.length() > MAX_CENTER_DISTANCE
        {
            continue;
        }

        let player_rotation = subtr_actor::quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        if local_ball_position.z > MAX_LOCAL_VERTICAL_OFFSET {
            continue;
        }

        let previous = find_counter(counters, player_id).unwrap_or(0);
        let counter_value = previous + 1;
        if let Some((_, value)) = counters.iter_mut().find(|(id, _)| id == player_id) {
            *value = counter_value;
        } else {
            counters.push((player_id.clone(), counter_value));
        }
        events.push(DodgeRefreshedEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: player_id.clone(),
            is_team_0: player.is_team_0,
            counter_value,
        });
    }

    events
}

impl SaLiveEventGenerator {
    fn sync_active_demos(&mut self, time: f32, events: &[SaDemolishEvent]) -> Vec<DemoEventSample> {
        self.active_demos
            .retain(|demo| demo.expires_at + f32::EPSILON >= time);

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
            let expires_at = time + active_duration_seconds;
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

    fn frame_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        gameplay: &GameplayState,
        explicit_live_play: Option<LivePlayState>,
        explicit_events: &SaFrameEventSlices<'_>,
    ) -> (FrameEventsState, LivePlayState) {
        let demo_events = explicit_demolish_events(frame, explicit_events.demolishes);
        let active_demos = self.sync_active_demos(frame.time, explicit_events.demolishes);
        let boost_pad_events = explicit_boost_pad_events(frame, explicit_events.boost_pad_events);
        let player_stat_events =
            explicit_player_stat_events(frame, explicit_events.player_stat_events);
        let goal_events = explicit_goal_events(frame, explicit_events.goals);
        let base_events = FrameEventsState {
            active_demos,
            demo_events,
            boost_pad_events,
            player_stat_events,
            goal_events,
            ..FrameEventsState::default()
        };
        let live_play = explicit_live_play
            .unwrap_or_else(|| self.live_play_tracker.state_parts(gameplay, &base_events));

        let empty_events = FrameEventsState::default();
        let touch_state = self
            .touch_state
            .update(frame, ball, players, &empty_events, &live_play);
        let mut touch_events = explicit_touch_events(frame, explicit_events.touches);
        if touch_events.is_empty() {
            touch_events.extend(touch_state.touch_events);
        }
        let inferred_dodge_refreshed_events = infer_dodge_refreshed_events(
            frame,
            ball,
            players,
            &touch_events,
            &mut self.dodge_refresh_counters,
        );
        let explicit_dodge_refreshed_events =
            explicit_dodge_refreshed_events(frame, explicit_events.dodge_refreshes);
        let explicit_dodge_refresh_keys = explicit_dodge_refreshed_events
            .iter()
            .map(|event| (event.player.clone(), event.frame))
            .collect::<HashSet<_>>();
        let mut dodge_refreshed_events = explicit_dodge_refreshed_events;
        dodge_refreshed_events.extend(inferred_dodge_refreshed_events.into_iter().filter(
            |event| !explicit_dodge_refresh_keys.contains(&(event.player.clone(), event.frame)),
        ));
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

fn frame_input(
    engine: &mut SaEngine,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    let frame_info = frame_info(frame);
    let ball = ball_state(frame);
    let players = player_state(sampled_players);
    let gameplay = gameplay_state(frame, sampled_players);
    let explicit_live_play = explicit_live_play_state(frame);
    let (frame_events, live_play) = engine.live_events.frame_events(
        &frame_info,
        &ball,
        &players,
        &gameplay,
        explicit_live_play,
        explicit_events,
    );
    let replay_meta = engine.live_replay_meta.as_ref();
    let processor = SaLiveProcessorView::new(replay_meta, frame, sampled_players, frame_events);
    FrameInput::timeline_with_live_play_state(
        &processor,
        frame.frame_number as usize,
        frame.time,
        frame.dt,
        live_play,
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

fn mechanic_start(event: &MechanicEvent) -> (usize, f32) {
    match event.timing {
        MechanicTiming::Moment { frame, time } => (frame, time),
        MechanicTiming::Span {
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
    mechanics: &[MechanicEvent],
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
    for (index, event) in timeline.iter().enumerate() {
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "timeline:{:?}:{:.3}:{}:{index}",
                    event.kind,
                    event.time,
                    player_index(player_id)
                ),
                kind: timeline_event_kind(event.kind),
                player_id: player_id.clone(),
                is_team_0,
                frame_number: event.frame.unwrap_or(0),
                time: event.time,
                confidence: 1.0,
            },
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

fn refresh_timeline_graph_views(engine: &mut SaEngine) {
    let Some(events) = engine
        .graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
    else {
        engine.events_json.clear();
        engine.frame_json.clear();
        engine.timeline_json.clear();
        engine.stats_json.clear();
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
    engine.events_json = serde_json::to_vec(&events).unwrap_or_default();

    if let Some(frame) = current_timeline_frame(&engine.graph) {
        record_timeline_frame(&mut engine.timeline_frames, frame.clone());
        engine.frame_json = serde_json::to_vec(&frame).unwrap_or_default();
    } else {
        engine.frame_json.clear();
    }
    engine.timeline_json = serialize_live_timeline(
        engine.live_replay_meta.as_ref(),
        events,
        engine.timeline_frames.clone(),
    );
    engine.stats_json = serialize_stats_graph_snapshot(engine);
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
    refresh_timeline_graph_views(engine);
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
    let Ok(explicit_events) = frame_event_slices(frame) else {
        return -1;
    };
    if sync_live_replay_meta(engine, players).is_err() {
        return -2;
    }
    let frame_input = frame_input(engine, frame, players, &explicit_events);
    if engine.graph.evaluate_with_state(&frame_input).is_err() {
        return -2;
    }

    refresh_timeline_graph_views(engine);
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
        .map(|engine| engine.events_json.len())
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

    let count = max_bytes.min(engine.events_json.len());
    ptr::copy_nonoverlapping(engine.events_json.as_ptr(), out_bytes, count);
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
        .map(|engine| engine.frame_json.len())
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

    let count = max_bytes.min(engine.frame_json.len());
    ptr::copy_nonoverlapping(engine.frame_json.as_ptr(), out_bytes, count);
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
        .map(|engine| engine.timeline_json.len())
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

    let count = max_bytes.min(engine.timeline_json.len());
    ptr::copy_nonoverlapping(engine.timeline_json.as_ptr(), out_bytes, count);
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
        .map(|engine| engine.stats_json.len())
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

    let count = max_bytes.min(engine.stats_json.len());
    ptr::copy_nonoverlapping(engine.stats_json.as_ptr(), out_bytes, count);
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
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use subtr_actor::{
        BoostPickupActivity, BoostPickupComparison, BoostPickupFieldHalf, BoostPickupPadType,
        DemoCalculator,
    };

    fn checked_in_header_text() -> String {
        let header_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("include")
            .join("subtr_actor_bakkesmod.h");
        std::fs::read_to_string(&header_path)
            .unwrap_or_else(|_| panic!("failed to read {}", header_path.display()))
    }

    fn header_enum_values(enum_name: &str) -> BTreeMap<String, i32> {
        let header = checked_in_header_text();
        let start = format!("typedef enum {enum_name} {{");
        let end = format!("}} {enum_name};");
        let mut in_enum = false;
        let mut values = BTreeMap::new();
        for line in header.lines() {
            let line = line.trim();
            if line == start {
                in_enum = true;
                continue;
            }
            if in_enum && line == end {
                return values;
            }
            if !in_enum || line.is_empty() {
                continue;
            }

            let line = line.trim_end_matches(',');
            let Some((name, value)) = line.split_once(" = ") else {
                continue;
            };
            values.insert(
                name.to_owned(),
                value
                    .parse::<i32>()
                    .unwrap_or_else(|_| panic!("invalid enum value in {enum_name}: {line}")),
            );
        }
        panic!("did not find enum {enum_name} in checked-in header");
    }

    fn header_struct_fields(struct_name: &str) -> Vec<String> {
        header_struct_field_declarations(struct_name)
            .into_iter()
            .map(|(_, field)| field)
            .collect()
    }

    fn header_struct_field_declarations(struct_name: &str) -> Vec<(String, String)> {
        let header = checked_in_header_text();
        let start = format!("typedef struct {struct_name} {{");
        let end = format!("}} {struct_name};");
        let mut in_struct = false;
        let mut fields = Vec::new();
        for line in header.lines() {
            let line = line.trim();
            if line == start {
                in_struct = true;
                continue;
            }
            if in_struct && line == end {
                return fields;
            }
            if !in_struct || line.is_empty() {
                continue;
            }

            let line = line.trim_end_matches(';');
            let Some((field_type, field)) = line.rsplit_once(' ') else {
                continue;
            };
            let pointer_prefix = field
                .chars()
                .take_while(|character| *character == '*')
                .collect::<String>();
            let field_type = if pointer_prefix.is_empty() {
                field_type.to_owned()
            } else {
                format!("{field_type} {pointer_prefix}")
            };
            fields.push((field_type, field.trim_start_matches('*').to_owned()));
        }
        panic!("did not find struct {struct_name} in checked-in header");
    }

    fn rust_struct_fields(struct_name: &str) -> Vec<String> {
        let source = include_str!("lib.rs");
        let start = format!("pub struct {struct_name} {{");
        let mut in_struct = false;
        let mut fields = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            if line == start {
                in_struct = true;
                continue;
            }
            if in_struct && line == "}" {
                return fields;
            }
            if !in_struct || line.is_empty() {
                continue;
            }

            let Some(field) = line.strip_prefix("pub ") else {
                continue;
            };
            let Some((name, _)) = field.split_once(':') else {
                continue;
            };
            fields.push(name.to_owned());
        }
        panic!("did not find struct {struct_name} in Rust source");
    }

    fn header_exported_function_names() -> BTreeSet<String> {
        checked_in_header_text()
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                let start = line.find("subtr_actor_bakkesmod_")?;
                let rest = &line[start..];
                let end = rest.find('(')?;
                Some(rest[..end].to_owned())
            })
            .collect()
    }

    fn rust_exported_function_names() -> BTreeSet<String> {
        include_str!("lib.rs")
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if !line.starts_with("pub ") || !line.contains(" extern \"C\" fn ") {
                    return None;
                }
                let (_, rest) = line.split_once("fn ")?;
                let end = rest.find('(')?;
                let name = &rest[..end];
                name.starts_with("subtr_actor_bakkesmod_")
                    .then(|| name.to_owned())
            })
            .collect()
    }

    #[test]
    fn checked_in_header_matches_event_abi_enums() {
        assert_eq!(
            header_enum_values("SaBoostPadEventKind"),
            BTreeMap::from([
                (
                    "SaBoostPadEventKindPickedUp".to_owned(),
                    SaBoostPadEventKind::PickedUp as i32,
                ),
                (
                    "SaBoostPadEventKindAvailable".to_owned(),
                    SaBoostPadEventKind::Available as i32,
                ),
            ])
        );
        assert_eq!(
            header_enum_values("SaPlayerStatEventKind"),
            BTreeMap::from([
                (
                    "SaPlayerStatEventKindShot".to_owned(),
                    SaPlayerStatEventKind::Shot as i32,
                ),
                (
                    "SaPlayerStatEventKindSave".to_owned(),
                    SaPlayerStatEventKind::Save as i32,
                ),
                (
                    "SaPlayerStatEventKindAssist".to_owned(),
                    SaPlayerStatEventKind::Assist as i32,
                ),
            ])
        );
        assert_eq!(
            header_enum_values("SaMechanicKind"),
            BTreeMap::from([
                (
                    "SaMechanicKindSpeedFlip".to_owned(),
                    SaMechanicKind::SpeedFlip as i32,
                ),
                (
                    "SaMechanicKindHalfFlip".to_owned(),
                    SaMechanicKind::HalfFlip as i32,
                ),
                (
                    "SaMechanicKindWavedash".to_owned(),
                    SaMechanicKind::Wavedash as i32,
                ),
                (
                    "SaMechanicKindBallCarry".to_owned(),
                    SaMechanicKind::BallCarry as i32,
                ),
                (
                    "SaMechanicKindAirDribble".to_owned(),
                    SaMechanicKind::AirDribble as i32,
                ),
                (
                    "SaMechanicKindCeilingShot".to_owned(),
                    SaMechanicKind::CeilingShot as i32,
                ),
                (
                    "SaMechanicKindWallAerial".to_owned(),
                    SaMechanicKind::WallAerial as i32,
                ),
                (
                    "SaMechanicKindWallAerialShot".to_owned(),
                    SaMechanicKind::WallAerialShot as i32,
                ),
                (
                    "SaMechanicKindCenter".to_owned(),
                    SaMechanicKind::Center as i32,
                ),
                (
                    "SaMechanicKindFlipReset".to_owned(),
                    SaMechanicKind::FlipReset as i32,
                ),
                (
                    "SaMechanicKindDoubleTap".to_owned(),
                    SaMechanicKind::DoubleTap as i32,
                ),
                (
                    "SaMechanicKindFlick".to_owned(),
                    SaMechanicKind::Flick as i32,
                ),
                (
                    "SaMechanicKindMustyFlick".to_owned(),
                    SaMechanicKind::MustyFlick as i32,
                ),
                (
                    "SaMechanicKindOneTimer".to_owned(),
                    SaMechanicKind::OneTimer as i32,
                ),
                ("SaMechanicKindPass".to_owned(), SaMechanicKind::Pass as i32),
                (
                    "SaMechanicKindHalfVolley".to_owned(),
                    SaMechanicKind::HalfVolley as i32,
                ),
                (
                    "SaMechanicKindWhiff".to_owned(),
                    SaMechanicKind::Whiff as i32,
                ),
                ("SaMechanicKindBump".to_owned(), SaMechanicKind::Bump as i32),
                (
                    "SaMechanicKindBackboard".to_owned(),
                    SaMechanicKind::Backboard as i32,
                ),
                (
                    "SaMechanicKindBoostPickup".to_owned(),
                    SaMechanicKind::BoostPickup as i32,
                ),
                ("SaMechanicKindDemo".to_owned(), SaMechanicKind::Demo as i32),
                (
                    "SaMechanicKindFiftyFifty".to_owned(),
                    SaMechanicKind::FiftyFifty as i32,
                ),
                (
                    "SaMechanicKindAerialGoal".to_owned(),
                    SaMechanicKind::AerialGoal as i32,
                ),
                (
                    "SaMechanicKindHighAerialGoal".to_owned(),
                    SaMechanicKind::HighAerialGoal as i32,
                ),
                (
                    "SaMechanicKindLongDistanceGoal".to_owned(),
                    SaMechanicKind::LongDistanceGoal as i32,
                ),
                (
                    "SaMechanicKindOwnHalfGoal".to_owned(),
                    SaMechanicKind::OwnHalfGoal as i32,
                ),
                (
                    "SaMechanicKindEmptyNetGoal".to_owned(),
                    SaMechanicKind::EmptyNetGoal as i32,
                ),
                (
                    "SaMechanicKindCounterAttackGoal".to_owned(),
                    SaMechanicKind::CounterAttackGoal as i32,
                ),
                (
                    "SaMechanicKindFlickGoal".to_owned(),
                    SaMechanicKind::FlickGoal as i32,
                ),
                (
                    "SaMechanicKindDoubleTapGoal".to_owned(),
                    SaMechanicKind::DoubleTapGoal as i32,
                ),
                (
                    "SaMechanicKindOneTimerGoal".to_owned(),
                    SaMechanicKind::OneTimerGoal as i32,
                ),
                (
                    "SaMechanicKindAirDribbleGoal".to_owned(),
                    SaMechanicKind::AirDribbleGoal as i32,
                ),
                (
                    "SaMechanicKindFlipResetGoal".to_owned(),
                    SaMechanicKind::FlipResetGoal as i32,
                ),
                (
                    "SaMechanicKindHalfVolleyGoal".to_owned(),
                    SaMechanicKind::HalfVolleyGoal as i32,
                ),
                ("SaMechanicKindGoal".to_owned(), SaMechanicKind::Goal as i32),
                ("SaMechanicKindShot".to_owned(), SaMechanicKind::Shot as i32),
                ("SaMechanicKindSave".to_owned(), SaMechanicKind::Save as i32),
                (
                    "SaMechanicKindAssist".to_owned(),
                    SaMechanicKind::Assist as i32,
                ),
                (
                    "SaMechanicKindDeath".to_owned(),
                    SaMechanicKind::Death as i32,
                ),
            ])
        );
        assert_eq!(
            header_enum_values("SaTeamEventKind"),
            BTreeMap::from([(
                "SaTeamEventKindRush".to_owned(),
                SaTeamEventKind::Rush as i32,
            )])
        );
        assert_eq!(
            header_enum_values("SaGoalBuildupKind"),
            BTreeMap::from([
                (
                    "SaGoalBuildupKindCounterAttack".to_owned(),
                    SaGoalBuildupKind::CounterAttack as i32,
                ),
                (
                    "SaGoalBuildupKindSustainedPressure".to_owned(),
                    SaGoalBuildupKind::SustainedPressure as i32,
                ),
                (
                    "SaGoalBuildupKindOther".to_owned(),
                    SaGoalBuildupKind::Other as i32,
                ),
            ])
        );
    }

    #[test]
    fn checked_in_header_declares_every_exported_function() {
        assert_eq!(
            header_exported_function_names(),
            rust_exported_function_names()
        );
    }

    #[test]
    fn checked_in_header_matches_event_abi_struct_fields() {
        for struct_name in [
            "SaVec3",
            "SaQuat",
            "SaRigidBody",
            "SaPlayerFrame",
            "SaTouchEvent",
            "SaDodgeRefreshedEvent",
            "SaBoostPadEvent",
            "SaGoalEvent",
            "SaPlayerStatEvent",
            "SaDemolishEvent",
            "SaLiveFrame",
            "SaMechanicEvent",
            "SaTeamEvent",
            "SaGoalContextEvent",
        ] {
            assert_eq!(
                header_struct_fields(struct_name),
                rust_struct_fields(struct_name),
                "checked-in header field order should match Rust repr(C) struct {struct_name}"
            );
        }
    }

    #[test]
    fn checked_in_header_matches_event_abi_struct_field_types() {
        let expected = BTreeMap::from([
            (
                "SaVec3",
                vec![("float", "x"), ("float", "y"), ("float", "z")],
            ),
            (
                "SaQuat",
                vec![
                    ("float", "x"),
                    ("float", "y"),
                    ("float", "z"),
                    ("float", "w"),
                ],
            ),
            (
                "SaRigidBody",
                vec![
                    ("SaVec3", "location"),
                    ("SaQuat", "rotation"),
                    ("SaVec3", "linear_velocity"),
                    ("SaVec3", "angular_velocity"),
                    ("uint8_t", "has_linear_velocity"),
                    ("uint8_t", "has_angular_velocity"),
                    ("uint8_t", "sleeping"),
                ],
            ),
            (
                "SaPlayerFrame",
                vec![
                    ("uint32_t", "player_index"),
                    ("const char *", "player_name"),
                    ("uint8_t", "is_team_0"),
                    ("uint8_t", "has_rigid_body"),
                    ("SaRigidBody", "rigid_body"),
                    ("float", "boost_amount"),
                    ("float", "last_boost_amount"),
                    ("uint8_t", "boost_active"),
                    ("uint8_t", "jump_active"),
                    ("uint8_t", "double_jump_active"),
                    ("uint8_t", "dodge_active"),
                    ("uint8_t", "powerslide_active"),
                    ("uint8_t", "has_match_stats"),
                    ("int32_t", "match_goals"),
                    ("int32_t", "match_assists"),
                    ("int32_t", "match_saves"),
                    ("int32_t", "match_shots"),
                    ("int32_t", "match_score"),
                ],
            ),
            (
                "SaTouchEvent",
                vec![
                    ("uint32_t", "player_index"),
                    ("uint8_t", "has_player"),
                    ("uint8_t", "is_team_0"),
                    ("float", "closest_approach_distance"),
                    ("uint8_t", "has_closest_approach_distance"),
                ],
            ),
            (
                "SaDodgeRefreshedEvent",
                vec![
                    ("uint32_t", "player_index"),
                    ("uint8_t", "is_team_0"),
                    ("int32_t", "counter_value"),
                ],
            ),
            (
                "SaBoostPadEvent",
                vec![
                    ("uint32_t", "pad_id"),
                    ("SaBoostPadEventKind", "kind"),
                    ("uint8_t", "sequence"),
                    ("uint32_t", "player_index"),
                    ("uint8_t", "has_player"),
                ],
            ),
            (
                "SaGoalEvent",
                vec![
                    ("uint8_t", "scoring_team_is_team_0"),
                    ("uint32_t", "player_index"),
                    ("uint8_t", "has_player"),
                    ("int32_t", "team_zero_score"),
                    ("uint8_t", "has_team_zero_score"),
                    ("int32_t", "team_one_score"),
                    ("uint8_t", "has_team_one_score"),
                ],
            ),
            (
                "SaPlayerStatEvent",
                vec![
                    ("uint32_t", "player_index"),
                    ("uint8_t", "is_team_0"),
                    ("SaPlayerStatEventKind", "kind"),
                    ("uint8_t", "has_shot_ball"),
                    ("SaRigidBody", "shot_ball"),
                    ("uint8_t", "has_shot_player"),
                    ("SaRigidBody", "shot_player"),
                ],
            ),
            (
                "SaDemolishEvent",
                vec![
                    ("uint32_t", "attacker_index"),
                    ("uint32_t", "victim_index"),
                    ("SaVec3", "attacker_velocity"),
                    ("SaVec3", "victim_velocity"),
                    ("SaVec3", "victim_location"),
                    ("float", "active_duration_seconds"),
                ],
            ),
            (
                "SaLiveFrame",
                vec![
                    ("uint64_t", "frame_number"),
                    ("float", "time"),
                    ("float", "dt"),
                    ("int32_t", "seconds_remaining"),
                    ("uint8_t", "has_seconds_remaining"),
                    ("int32_t", "game_state"),
                    ("uint8_t", "has_game_state"),
                    ("int32_t", "kickoff_countdown_time"),
                    ("uint8_t", "has_kickoff_countdown_time"),
                    ("uint8_t", "ball_has_been_hit"),
                    ("uint8_t", "has_ball_has_been_hit"),
                    ("int32_t", "team_zero_score"),
                    ("uint8_t", "has_team_zero_score"),
                    ("int32_t", "team_one_score"),
                    ("uint8_t", "has_team_one_score"),
                    ("uint8_t", "possession_team_is_team_0"),
                    ("uint8_t", "has_possession_team"),
                    ("uint8_t", "scored_on_team_is_team_0"),
                    ("uint8_t", "has_scored_on_team"),
                    ("uint8_t", "live_play"),
                    ("uint8_t", "has_live_play"),
                    ("uint8_t", "has_ball"),
                    ("SaRigidBody", "ball"),
                    ("const SaPlayerFrame *", "players"),
                    ("size_t", "player_count"),
                    ("const SaTouchEvent *", "touches"),
                    ("size_t", "touch_count"),
                    ("const SaDodgeRefreshedEvent *", "dodge_refreshes"),
                    ("size_t", "dodge_refresh_count"),
                    ("const SaBoostPadEvent *", "boost_pad_events"),
                    ("size_t", "boost_pad_event_count"),
                    ("const SaGoalEvent *", "goals"),
                    ("size_t", "goal_count"),
                    ("const SaPlayerStatEvent *", "player_stat_events"),
                    ("size_t", "player_stat_event_count"),
                    ("const SaDemolishEvent *", "demolishes"),
                    ("size_t", "demolish_count"),
                ],
            ),
            (
                "SaMechanicEvent",
                vec![
                    ("SaMechanicKind", "kind"),
                    ("uint32_t", "player_index"),
                    ("uint8_t", "is_team_0"),
                    ("uint64_t", "frame_number"),
                    ("float", "time"),
                    ("float", "confidence"),
                ],
            ),
            (
                "SaTeamEvent",
                vec![
                    ("SaTeamEventKind", "kind"),
                    ("uint8_t", "is_team_0"),
                    ("uint64_t", "start_frame"),
                    ("uint64_t", "end_frame"),
                    ("float", "start_time"),
                    ("float", "end_time"),
                    ("uint32_t", "attackers"),
                    ("uint32_t", "defenders"),
                    ("float", "confidence"),
                ],
            ),
            (
                "SaGoalContextEvent",
                vec![
                    ("uint64_t", "frame_number"),
                    ("float", "time"),
                    ("uint8_t", "scoring_team_is_team_0"),
                    ("uint8_t", "has_scorer"),
                    ("uint32_t", "scorer_index"),
                    ("uint8_t", "has_scoring_team_most_back_player"),
                    ("uint32_t", "scoring_team_most_back_player_index"),
                    ("uint8_t", "has_defending_team_most_back_player"),
                    ("uint32_t", "defending_team_most_back_player_index"),
                    ("uint8_t", "has_ball_position"),
                    ("SaVec3", "ball_position"),
                    ("uint8_t", "has_ball_air_time_before_goal"),
                    ("float", "ball_air_time_before_goal"),
                    ("SaGoalBuildupKind", "goal_buildup"),
                ],
            ),
        ]);

        for (struct_name, expected_fields) in expected {
            let expected_fields = expected_fields
                .into_iter()
                .map(|(field_type, field)| (field_type.to_owned(), field.to_owned()))
                .collect::<Vec<_>>();
            assert_eq!(
                header_struct_field_declarations(struct_name),
                expected_fields,
                "checked-in header field types should match the intended C ABI for {struct_name}"
            );
        }
    }

    fn rigid_body(location: SaVec3, linear_velocity: SaVec3) -> SaRigidBody {
        SaRigidBody {
            location,
            rotation: SaQuat::default(),
            linear_velocity,
            angular_velocity: SaVec3::default(),
            has_linear_velocity: 1,
            has_angular_velocity: 1,
            sleeping: 0,
        }
    }

    fn live_frame(frame_number: u64, ball: SaRigidBody, players: &[SaPlayerFrame]) -> SaLiveFrame {
        SaLiveFrame {
            frame_number,
            time: frame_number as f32 * 0.1,
            dt: 0.1,
            seconds_remaining: 299,
            has_seconds_remaining: 1,
            game_state: 0,
            has_game_state: 0,
            kickoff_countdown_time: 0,
            has_kickoff_countdown_time: 0,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            has_ball: 1,
            ball,
            players: players.as_ptr(),
            player_count: players.len(),
            ..SaLiveFrame::default()
        }
    }

    fn player_at_index(player_index: u32, is_team_0: bool, location: SaVec3) -> SaPlayerFrame {
        SaPlayerFrame {
            player_index,
            player_name: ptr::null(),
            is_team_0: is_team_0 as u8,
            has_rigid_body: 1,
            rigid_body: rigid_body(location, SaVec3::default()),
            boost_amount: 33.0,
            last_boost_amount: 33.0,
            boost_active: 0,
            jump_active: 0,
            double_jump_active: 0,
            dodge_active: 0,
            powerslide_active: 0,
            has_match_stats: 1,
            match_goals: player_index as i32,
            match_assists: player_index as i32 + 1,
            match_saves: player_index as i32 + 2,
            match_shots: player_index as i32 + 3,
            match_score: player_index as i32 + 100,
        }
    }

    fn player_at(location: SaVec3) -> SaPlayerFrame {
        player_at_index(0, true, location)
    }

    fn normalized_mechanic(id: &str, kind: &str, frame: usize, time: f32) -> MechanicEvent {
        MechanicEvent {
            id: id.to_owned(),
            kind: kind.to_owned(),
            player_id: RemoteId::SplitScreen(0),
            is_team_0: true,
            timing: MechanicTiming::Moment { frame, time },
            properties: Vec::new(),
        }
    }

    fn whiff_event(frame: usize, time: f32, player_index: u32) -> WhiffEvent {
        WhiffEvent {
            kind: WhiffEventKind::Whiff,
            time,
            frame,
            player: RemoteId::SplitScreen(player_index),
            is_team_0: player_index == 0,
            closest_approach_distance: 42.0,
            forward_alignment: 0.7,
            approach_speed: 900.0,
            dodge_active: false,
            aerial: false,
        }
    }

    fn bump_event(frame: usize, time: f32, confidence: f32) -> BumpEvent {
        BumpEvent {
            time,
            frame,
            initiator: RemoteId::SplitScreen(0),
            victim: RemoteId::SplitScreen(1),
            initiator_is_team_0: true,
            victim_is_team_0: false,
            is_team_bump: false,
            strength: 800.0,
            confidence,
            contact_distance: 120.0,
            closing_speed: 500.0,
            victim_impulse: 220.0,
            initiator_position: [0.0, 0.0, 0.0],
            victim_position: [100.0, 0.0, 0.0],
        }
    }

    fn backboard_event(frame: usize, time: f32) -> BackboardBounceEvent {
        BackboardBounceEvent {
            time,
            frame,
            player: RemoteId::SplitScreen(0),
            is_team_0: true,
        }
    }

    fn boost_pickup_event(frame: usize, time: f32) -> BoostPickupComparisonEvent {
        BoostPickupComparisonEvent {
            comparison: BoostPickupComparison::Both,
            frame,
            time,
            player_id: RemoteId::SplitScreen(0),
            is_team_0: true,
            pad_type: BoostPickupPadType::Big,
            field_half: BoostPickupFieldHalf::Opponent,
            activity: BoostPickupActivity::Active,
            reported_frame: Some(frame),
            reported_time: Some(time),
            inferred_frame: None,
            inferred_time: None,
            boost_before: Some(20.0),
            boost_after: Some(100.0),
        }
    }

    fn fifty_fifty_event(
        start_frame: usize,
        resolve_frame: usize,
        resolve_time: f32,
    ) -> FiftyFiftyEvent {
        FiftyFiftyEvent {
            start_time: 1.0,
            start_frame,
            resolve_time,
            resolve_frame,
            is_kickoff: false,
            team_zero_player: Some(RemoteId::SplitScreen(0)),
            team_one_player: Some(RemoteId::SplitScreen(1)),
            team_zero_position: [0.0, 0.0, 0.0],
            team_one_position: [100.0, 0.0, 0.0],
            midpoint: [50.0, 0.0, 0.0],
            plane_normal: [1.0, 0.0, 0.0],
            winning_team_is_team_0: Some(false),
            possession_team_is_team_0: Some(false),
        }
    }

    fn goal_tag_event(kind: GoalTagKind, scorer: Option<RemoteId>) -> GoalTagEvent {
        GoalTagEvent {
            goal_index: 0,
            time: 1.36,
            frame: 13,
            kind,
            scoring_team_is_team_0: false,
            scorer,
            confidence: 0.72,
            modifiers: Vec::new(),
            evidence: Vec::new(),
        }
    }

    fn rush_event(
        start_frame: usize,
        end_frame: usize,
        end_time: f32,
        is_team_0: bool,
    ) -> RushEvent {
        RushEvent {
            start_time: 1.0,
            start_frame,
            end_time,
            end_frame,
            is_team_0,
            attackers: 3,
            defenders: 2,
        }
    }

    fn goal_context_event(frame: usize, time: f32) -> GoalContextEvent {
        GoalContextEvent {
            time,
            frame,
            scoring_team_is_team_0: false,
            scorer: Some(RemoteId::SplitScreen(1)),
            scoring_team_most_back_player: Some(RemoteId::SplitScreen(1)),
            defending_team_most_back_player: Some(RemoteId::SplitScreen(0)),
            ball_position: Some(subtr_actor::GoalContextPosition {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }),
            ball_air_time_before_goal: Some(1.25),
            goal_buildup: GoalBuildupKind::CounterAttack,
            scorer_last_touch: None,
            players: Vec::new(),
        }
    }

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
            ..SaLiveFrame::default()
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
            ..SaLiveFrame::default()
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, -1);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_updates_analysis_graph_state() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            seconds_remaining: 299,
            has_seconds_remaining: 1,
            game_state: 0,
            has_game_state: 0,
            kickoff_countdown_time: 0,
            has_kickoff_countdown_time: 0,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            team_zero_score: 2,
            has_team_zero_score: 1,
            team_one_score: 1,
            has_team_one_score: 1,
            possession_team_is_team_0: 1,
            has_possession_team: 1,
            scored_on_team_is_team_0: 0,
            has_scored_on_team: 1,
            live_play: 1,
            has_ball: 0,
            ball: SaRigidBody::default(),
            players: ptr::null(),
            player_count: 0,
            ..SaLiveFrame::default()
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, 0);
        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        assert!(engine_ref.live_replay_meta_initialized);
        assert!(engine_ref.live_replay_meta.is_some());
        let frame_info = engine_ref
            .graph
            .state::<FrameInfo>()
            .expect("full analysis graph should expose frame info state");
        engine_ref
            .graph
            .state::<StatsTimelineEventsState>()
            .expect("live graph should expose normalized timeline events state");
        assert_eq!(frame_info.frame_number, 7);
        assert_eq!(frame_info.seconds_remaining, Some(299));
        let gameplay = engine_ref
            .graph
            .state::<GameplayState>()
            .expect("full analysis graph should expose gameplay state");
        assert_eq!(gameplay.current_score(), Some((2, 1)));
        assert_eq!(gameplay.possession_team_is_team_0, Some(true));
        assert_eq!(gameplay.scored_on_team_is_team_0, Some(false));
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn live_graph_contains_every_shared_analysis_node() {
        let mut expected_graph = graph_with_all_analysis_nodes();
        expected_graph
            .resolve()
            .expect("shared graph should resolve");
        let expected_names = expected_graph.node_names().collect::<HashSet<_>>();

        let mut graph = live_analysis_graph();
        graph.resolve().expect("live graph should resolve");
        let live_names = graph.node_names().collect::<HashSet<_>>();
        let builtin_names = builtin_analysis_node_names()
            .iter()
            .copied()
            .collect::<HashSet<_>>();

        for name in expected_names {
            assert!(
                live_names.contains(name),
                "live graph should include shared analysis node {name}"
            );
        }
        for name in &live_names {
            assert!(
                builtin_names.contains(name),
                "live graph node should be callable by builtin name: {name}"
            );
        }
        assert!(live_names.contains("stats_timeline_frame"));
        assert!(live_names.contains("stats_timeline_events"));
    }

    #[test]
    fn process_frame_uses_explicit_live_play_state_for_analysis_graph() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 0,
            has_live_play: 1,
            ..SaLiveFrame::default()
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, 0);
        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let live_play = engine_ref
            .graph
            .state::<LivePlayState>()
            .expect("full analysis graph should expose live play state");
        assert_eq!(live_play.gameplay_phase, GameplayPhase::Unknown);
        assert!(!live_play.is_live_play);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_derives_live_play_when_not_explicit() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            ball_has_been_hit: 0,
            has_ball_has_been_hit: 1,
            live_play: 1,
            has_live_play: 0,
            ..SaLiveFrame::default()
        };

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, 0);
        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let live_play = engine_ref
            .graph
            .state::<LivePlayState>()
            .expect("full analysis graph should expose live play state");
        assert_eq!(
            live_play.gameplay_phase,
            GameplayPhase::KickoffWaitingForTouch
        );
        assert!(!live_play.is_live_play);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_treats_sampled_game_state_as_replay_phase_signal() {
        const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
        const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

        let engine = subtr_actor_bakkesmod_engine_create();
        let kickoff_frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            game_state: GAME_STATE_KICKOFF_COUNTDOWN,
            has_game_state: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            ..SaLiveFrame::default()
        };

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &kickoff_frame) },
            0
        );
        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let live_play = engine_ref
            .graph
            .state::<LivePlayState>()
            .expect("full analysis graph should expose live play state");
        assert_eq!(live_play.gameplay_phase, GameplayPhase::KickoffCountdown);
        assert!(!live_play.is_live_play);
        let gameplay = engine_ref
            .graph
            .state::<GameplayState>()
            .expect("full analysis graph should expose gameplay state");
        assert_eq!(gameplay.game_state, Some(GAME_STATE_KICKOFF_COUNTDOWN));

        let replay_frame = SaLiveFrame {
            frame_number: 8,
            time: 1.6,
            dt: 0.016,
            game_state: GAME_STATE_GOAL_SCORED_REPLAY,
            has_game_state: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            ..SaLiveFrame::default()
        };
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &replay_frame) },
            0
        );
        let live_play = engine_ref
            .graph
            .state::<LivePlayState>()
            .expect("full analysis graph should expose live play state");
        assert_eq!(live_play.gameplay_phase, GameplayPhase::PostGoal);
        assert!(!live_play.is_live_play);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn finish_refreshes_exported_graph_views() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            seconds_remaining: 299,
            has_seconds_remaining: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            has_live_play: 1,
            players: ptr::null(),
            player_count: 0,
            ..SaLiveFrame::default()
        };

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );
        assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
        assert!(unsafe { subtr_actor_bakkesmod_events_json_len(engine) } > 0);
        assert!(unsafe { subtr_actor_bakkesmod_frame_json_len(engine) } > 0);
        assert!(unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) } > 0);
        assert!(unsafe { subtr_actor_bakkesmod_stats_json_len(engine) } > 0);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn finish_drains_finalized_live_ball_carry_events() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let mut events = [SaMechanicEvent {
            kind: SaMechanicKind::SpeedFlip,
            player_index: 0,
            is_team_0: 0,
            frame_number: 0,
            time: 0.0,
            confidence: 0.0,
        }; 8];

        for frame_number in 1..=12 {
            let players = [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })];
            let mut frame = live_frame(
                frame_number,
                rigid_body(
                    SaVec3 {
                        x: frame_number as f32 * 20.0,
                        y: 0.0,
                        z: 120.0,
                    },
                    SaVec3::default(),
                ),
                &players,
            );
            frame.has_live_play = 1;
            if frame_number == 1 {
                let touches = [SaTouchEvent {
                    player_index: 0,
                    has_player: 1,
                    is_team_0: 1,
                    closest_approach_distance: 0.0,
                    has_closest_approach_distance: 1,
                }];
                frame.touches = touches.as_ptr();
                frame.touch_count = touches.len();
                assert_eq!(
                    unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
                    0
                );
            } else {
                assert_eq!(
                    unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
                    0
                );
            }
        }

        let pre_finish_count = unsafe {
            subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len())
        };
        assert!(pre_finish_count > 0);
        assert!(events[..pre_finish_count]
            .iter()
            .all(|event| event.kind != SaMechanicKind::BallCarry));
        assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
        let count = unsafe {
            subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len())
        };
        assert_eq!(count, 1);
        assert_eq!(events[0].kind, SaMechanicKind::BallCarry);
        assert_eq!(events[0].player_index, 0);
        assert_eq!(events[0].is_team_0, 1);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn finish_rejects_null_engine() {
        assert_eq!(unsafe { subtr_actor_bakkesmod_finish(ptr::null_mut()) }, -1);
    }

    #[test]
    fn drains_pending_team_events_through_abi() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let engine_ref = unsafe { engine.as_mut().expect("engine should be valid") };
        engine_ref.pending_team_events.push(SaTeamEvent {
            kind: SaTeamEventKind::Rush,
            is_team_0: 1,
            start_frame: 4,
            end_frame: 9,
            start_time: 0.4,
            end_time: 0.9,
            attackers: 3,
            defenders: 1,
            confidence: 1.0,
        });
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_pending_team_event_count(engine) },
            1
        );

        let mut events = [SaTeamEvent {
            kind: SaTeamEventKind::Rush,
            is_team_0: 0,
            start_frame: 0,
            end_frame: 0,
            start_time: 0.0,
            end_time: 0.0,
            attackers: 0,
            defenders: 0,
            confidence: 0.0,
        }];
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_drain_team_events(engine, events.as_mut_ptr(), 1) },
            1
        );
        assert_eq!(events[0].kind, SaTeamEventKind::Rush);
        assert_eq!(events[0].is_team_0, 1);
        assert_eq!(events[0].attackers, 3);
        assert_eq!(events[0].defenders, 1);
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_pending_team_event_count(engine) },
            0
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_drain_team_events(engine, ptr::null_mut(), 1) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn drains_pending_goal_context_events_through_abi() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let engine_ref = unsafe { engine.as_mut().expect("engine should be valid") };
        engine_ref
            .pending_goal_context_events
            .push(SaGoalContextEvent {
                frame_number: 9,
                time: 0.9,
                scoring_team_is_team_0: 0,
                has_scorer: 1,
                scorer_index: 1,
                has_scoring_team_most_back_player: 1,
                scoring_team_most_back_player_index: 1,
                has_defending_team_most_back_player: 1,
                defending_team_most_back_player_index: 0,
                has_ball_position: 1,
                ball_position: SaVec3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                has_ball_air_time_before_goal: 1,
                ball_air_time_before_goal: 1.25,
                goal_buildup: SaGoalBuildupKind::CounterAttack,
            });
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_pending_goal_context_event_count(engine) },
            1
        );

        let mut events = [SaGoalContextEvent {
            frame_number: 0,
            time: 0.0,
            scoring_team_is_team_0: 0,
            has_scorer: 0,
            scorer_index: 0,
            has_scoring_team_most_back_player: 0,
            scoring_team_most_back_player_index: 0,
            has_defending_team_most_back_player: 0,
            defending_team_most_back_player_index: 0,
            has_ball_position: 0,
            ball_position: SaVec3::default(),
            has_ball_air_time_before_goal: 0,
            ball_air_time_before_goal: 0.0,
            goal_buildup: SaGoalBuildupKind::Other,
        }];
        assert_eq!(
            unsafe {
                subtr_actor_bakkesmod_drain_goal_context_events(engine, events.as_mut_ptr(), 1)
            },
            1
        );
        assert_eq!(events[0].frame_number, 9);
        assert_eq!(events[0].scorer_index, 1);
        assert_eq!(events[0].goal_buildup, SaGoalBuildupKind::CounterAttack);
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_pending_goal_context_event_count(engine) },
            0
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_drain_goal_context_events(engine, ptr::null_mut(), 1) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn exposes_full_timeline_events_json_after_processing_frame() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let frame = SaLiveFrame {
            frame_number: 7,
            time: 1.5,
            dt: 0.016,
            seconds_remaining: 299,
            has_seconds_remaining: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            players: ptr::null(),
            player_count: 0,
            ..SaLiveFrame::default()
        };

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );
        let json_len = unsafe { subtr_actor_bakkesmod_events_json_len(engine) };
        assert!(json_len > 0);
        let mut bytes = vec![0; json_len];
        let written = unsafe {
            subtr_actor_bakkesmod_write_events_json(engine, bytes.as_mut_ptr(), bytes.len())
        };
        assert_eq!(written, json_len);

        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("events json should be valid");
        assert!(value.get("timeline").is_some());
        assert!(value.get("mechanics").is_some());
        assert!(value.get("goal_context").is_some());
        assert!(value.get("boost_pickups").is_some());
        assert!(value.get("bump").is_some());
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_write_events_json(engine, ptr::null_mut(), 10) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn live_timeline_event_fields_are_classified_for_drain_coverage() {
        let value = serde_json::to_value(ReplayStatsTimelineEvents::default())
            .expect("default events should serialize");
        let fields = value
            .as_object()
            .expect("events should serialize as an object")
            .keys()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let accounted_fields = HashSet::from([
            "timeline",
            "mechanics",
            "goal_context",
            "backboard",
            "ceiling_shot",
            "wall_aerial",
            "wall_aerial_shot",
            "center",
            "double_tap",
            "fifty_fifty",
            "one_timer",
            "pass",
            "goal_tags",
            "rush",
            "speed_flip",
            "half_flip",
            "half_volley",
            "wavedash",
            "whiff",
            "boost_pickups",
            "bump",
        ]);
        assert_eq!(
            fields, accounted_fields,
            "new timeline event fields need an explicit live drain/export decision"
        );
    }

    #[test]
    fn exposes_live_graph_info_json() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let json_len = unsafe { subtr_actor_bakkesmod_graph_info_json_len(engine) };
        assert!(json_len > 0);
        let mut bytes = vec![0; json_len];
        let written = unsafe {
            subtr_actor_bakkesmod_write_graph_info_json(engine, bytes.as_mut_ptr(), bytes.len())
        };
        assert_eq!(written, json_len);

        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("graph info json should be valid");
        assert!(value["dag"]
            .as_str()
            .expect("dag should be a string")
            .contains("stats_timeline_events"));
        let builtin_names = value["builtin_analysis_node_names"]
            .as_array()
            .expect("builtin names should be an array");
        assert!(builtin_names.iter().any(|name| name == "settings"));
        assert!(builtin_names
            .iter()
            .any(|name| name == "continuous_ball_control"));
        assert!(builtin_names.iter().any(|name| name == "air_dribble"));
        assert!(builtin_names.iter().any(|name| name == "frame_info"));
        assert!(builtin_names.iter().any(|name| name == "live_play"));
        assert!(builtin_names
            .iter()
            .any(|name| name == "stats_timeline_frame"));
        assert!(builtin_names
            .iter()
            .any(|name| name == "stats_timeline_events"));
        let builtin_aliases = value["builtin_analysis_node_aliases"]
            .as_array()
            .expect("builtin aliases should be an array");
        assert!(builtin_aliases
            .iter()
            .any(|alias| alias["alias"] == "core" && alias["node_name"] == "match_stats"));
        assert!(builtin_aliases
            .iter()
            .any(|alias| alias["alias"] == "air_dribble" && alias["node_name"] == "ball_carry"));
        let stats_module_names = value["builtin_stats_module_names"]
            .as_array()
            .expect("stats module names should be an array");
        assert_eq!(stats_module_names.len(), builtin_stats_module_names().len());
        for module_name in builtin_stats_module_names() {
            assert!(
                stats_module_names.iter().any(|name| name == module_name),
                "graph info should expose stats module {module_name}"
            );
        }
        let node_names = value["node_names"]
            .as_array()
            .expect("node names should be an array");
        assert!(node_names
            .iter()
            .any(|name| name == "continuous_ball_control"));
        assert!(node_names.iter().any(|name| name == "stats_timeline_frame"));
        assert!(node_names
            .iter()
            .any(|name| name == "stats_timeline_events"));
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_write_graph_info_json(engine, ptr::null_mut(), 10) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn exposes_full_stats_timeline_json_after_processing_frames() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let blue_name = std::ffi::CString::new("Blue Live").unwrap();
        let mut players = [player_at_index(
            0,
            true,
            SaVec3 {
                x: -100.0,
                y: -200.0,
                z: 92.75,
            },
        )];
        players[0].player_name = blue_name.as_ptr();

        for (frame_number, time) in [(9, 1.75), (10, 1.766)] {
            let frame = SaLiveFrame {
                frame_number,
                time,
                dt: 0.016,
                seconds_remaining: 298,
                has_seconds_remaining: 1,
                ball_has_been_hit: 1,
                has_ball_has_been_hit: 1,
                live_play: 1,
                players: players.as_ptr(),
                player_count: players.len(),
                ..SaLiveFrame::default()
            };
            assert_eq!(
                unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
                0
            );
        }

        let json_len = unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) };
        assert!(json_len > 0);
        let mut bytes = vec![0; json_len];
        let written = unsafe {
            subtr_actor_bakkesmod_write_timeline_json(engine, bytes.as_mut_ptr(), bytes.len())
        };
        assert_eq!(written, json_len);

        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("timeline json should be valid");
        assert!(value.get("config").is_some());
        assert!(value.get("events").is_some());
        assert_eq!(value["replay_meta"]["team_zero"][0]["name"], "Blue Live");
        let frames = value["frames"].as_array().expect("frames array");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0]["frame_number"], 9);
        assert_eq!(frames[1]["frame_number"], 10);
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_write_timeline_json(engine, ptr::null_mut(), 10) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn exposes_stats_collector_module_json_after_processing_frame() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let blue_name = std::ffi::CString::new("Blue Live").unwrap();
        let mut players = [player_at_index(
            0,
            true,
            SaVec3 {
                x: -100.0,
                y: -200.0,
                z: 92.75,
            },
        )];
        players[0].player_name = blue_name.as_ptr();
        let frame = SaLiveFrame {
            frame_number: 9,
            time: 1.75,
            dt: 0.016,
            seconds_remaining: 298,
            has_seconds_remaining: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            players: players.as_ptr(),
            player_count: players.len(),
            ..SaLiveFrame::default()
        };

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );
        let json_len = unsafe { subtr_actor_bakkesmod_stats_json_len(engine) };
        assert!(json_len > 0);
        let mut bytes = vec![0; json_len];
        let written = unsafe {
            subtr_actor_bakkesmod_write_stats_json(engine, bytes.as_mut_ptr(), bytes.len())
        };
        assert_eq!(written, json_len);

        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("stats json should be valid");
        let module_names = value["module_names"]
            .as_array()
            .expect("module names should be an array");
        assert_eq!(module_names.len(), builtin_stats_module_names().len());
        for module_name in builtin_stats_module_names() {
            assert!(
                module_names.iter().any(|name| name == module_name),
                "stats json should expose stats module {module_name}"
            );
            assert!(
                value["modules"].get(module_name).is_some(),
                "stats json should include module payload for {module_name}"
            );
        }
        assert!(value["config"].get("positioning").is_some());
        assert!(value["modules"].get("core").is_some());
        assert!(value["modules"].get("boost").is_some());
        assert!(value["modules"].get("demo").is_some());
        assert_eq!(value["frame"]["frame_number"], 9);
        assert_eq!(
            value["frame"]["modules"]["core"]["player_stats"]
                .as_array()
                .expect("core frame player stats should be an array")
                .len(),
            1
        );
        let frame_modules = value["frame"]["modules"]
            .as_object()
            .expect("frame modules should be an object");
        for module_name in builtin_stats_module_names() {
            assert!(
                frame_modules.contains_key(*module_name),
                "stats frame should include module payload for {module_name}"
            );
        }
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_write_stats_json(engine, ptr::null_mut(), 10) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn exposes_current_timeline_frame_json_after_processing_frame() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let blue_name = std::ffi::CString::new("Blue Live").unwrap();
        let orange_name = std::ffi::CString::new("Orange Live").unwrap();
        let mut players = [
            player_at_index(
                0,
                true,
                SaVec3 {
                    x: -100.0,
                    y: -200.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                1,
                false,
                SaVec3 {
                    x: 100.0,
                    y: 200.0,
                    z: 92.75,
                },
            ),
        ];
        players[0].player_name = blue_name.as_ptr();
        players[1].player_name = orange_name.as_ptr();
        let frame = SaLiveFrame {
            frame_number: 9,
            time: 1.75,
            dt: 0.016,
            seconds_remaining: 298,
            has_seconds_remaining: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            players: players.as_ptr(),
            player_count: players.len(),
            ..SaLiveFrame::default()
        };

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );
        let json_len = unsafe { subtr_actor_bakkesmod_frame_json_len(engine) };
        assert!(json_len > 0);
        let mut bytes = vec![0; json_len];
        let written = unsafe {
            subtr_actor_bakkesmod_write_frame_json(engine, bytes.as_mut_ptr(), bytes.len())
        };
        assert_eq!(written, json_len);

        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("frame json should be valid");
        assert_eq!(value["frame_number"], 9);
        assert_eq!(value["seconds_remaining"], 298);
        assert_eq!(value["gameplay_phase"], "active_play");
        assert_eq!(value["players"].as_array().expect("players array").len(), 2);
        assert_eq!(value["players"][0]["name"], "Blue Live");
        assert_eq!(value["players"][0]["is_team_0"], true);
        assert_eq!(value["players"][1]["name"], "Orange Live");
        assert_eq!(value["players"][1]["is_team_0"], false);
        assert!(value.get("team_zero").is_some());
        assert!(value.get("team_one").is_some());
        let team_module_names = [
            "fifty_fifty",
            "possession",
            "pressure",
            "rotation",
            "rush",
            "core",
            "backboard",
            "double_tap",
            "one_timer",
            "pass",
            "ball_carry",
            "air_dribble",
            "boost",
            "bump",
            "half_volley",
            "movement",
            "powerslide",
            "demo",
        ];
        let player_module_names = [
            "core",
            "backboard",
            "ceiling_shot",
            "wall_aerial",
            "wall_aerial_shot",
            "double_tap",
            "one_timer",
            "pass",
            "fifty_fifty",
            "speed_flip",
            "half_flip",
            "half_volley",
            "wavedash",
            "touch",
            "whiff",
            "flick",
            "musty_flick",
            "dodge_reset",
            "ball_carry",
            "air_dribble",
            "boost",
            "bump",
            "movement",
            "positioning",
            "rotation",
            "powerslide",
            "demo",
        ];
        for module_name in team_module_names {
            assert!(
                value["team_zero"].get(module_name).is_some(),
                "typed stats frame should include team_zero.{module_name}"
            );
            assert!(
                value["team_one"].get(module_name).is_some(),
                "typed stats frame should include team_one.{module_name}"
            );
        }
        for module_name in player_module_names {
            assert!(
                value["players"][0].get(module_name).is_some(),
                "typed stats frame should include player module {module_name}"
            );
        }
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_write_frame_json(engine, ptr::null_mut(), 10) },
            0
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_generates_live_touch_events_for_graph_input() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [player_at(SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 92.75,
        })];
        let first = live_frame(
            1,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
                SaVec3::default(),
            ),
            &players,
        );
        let second = live_frame(
            2,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
                SaVec3 {
                    x: 300.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            &players,
        );

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
            0
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
            0
        );

        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        assert_eq!(frame_events.touch_events.len(), 1);
        assert_eq!(frame_events.touch_events[0].frame, 2);
        assert_eq!(
            frame_events.touch_events[0].player,
            Some(RemoteId::SplitScreen(0))
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_generates_live_dodge_refreshed_events_for_airborne_ball_touches() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [player_at(SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 180.0,
        })];
        let first = live_frame(
            1,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 180.0,
                },
                SaVec3::default(),
            ),
            &players,
        );
        let second = live_frame(
            2,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 180.0,
                },
                SaVec3 {
                    x: 300.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            &players,
        );

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
            0
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
            0
        );

        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
        assert_eq!(
            frame_events.dodge_refreshed_events[0].player,
            RemoteId::SplitScreen(0)
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn explicit_dodge_refreshed_events_suppress_inferred_duplicates() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [player_at(SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 180.0,
        })];
        let touches = [SaTouchEvent {
            player_index: 0,
            has_player: 1,
            is_team_0: 1,
            closest_approach_distance: 10.0,
            has_closest_approach_distance: 1,
        }];
        let dodge_refreshes = [SaDodgeRefreshedEvent {
            player_index: 0,
            is_team_0: 1,
            counter_value: 7,
        }];
        let mut frame = live_frame(
            1,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 180.0,
                },
                SaVec3 {
                    x: 300.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            &players,
        );
        frame.touches = touches.as_ptr();
        frame.touch_count = touches.len();
        frame.dodge_refreshes = dodge_refreshes.as_ptr();
        frame.dodge_refresh_count = dodge_refreshes.len();

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );

        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
        assert_eq!(frame_events.dodge_refreshed_events[0].counter_value, 7);
        assert_eq!(
            frame_events.dodge_refreshed_events[0].player,
            RemoteId::SplitScreen(0)
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_accepts_explicit_live_event_arrays_for_graph_input() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [
            player_at_index(
                0,
                true,
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                1,
                false,
                SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
        ];
        let touches = [SaTouchEvent {
            player_index: 0,
            has_player: 1,
            is_team_0: 1,
            closest_approach_distance: 12.0,
            has_closest_approach_distance: 1,
        }];
        let dodge_refreshes = [SaDodgeRefreshedEvent {
            player_index: 0,
            is_team_0: 1,
            counter_value: 3,
        }];
        let boost_pad_events = [SaBoostPadEvent {
            pad_id: 34,
            kind: SaBoostPadEventKind::PickedUp,
            sequence: 2,
            player_index: 0,
            has_player: 1,
        }];
        let goals = [SaGoalEvent {
            scoring_team_is_team_0: 1,
            player_index: 0,
            has_player: 1,
            team_zero_score: 1,
            has_team_zero_score: 1,
            team_one_score: 0,
            has_team_one_score: 1,
        }];
        let player_stat_events = [SaPlayerStatEvent {
            player_index: 0,
            is_team_0: 1,
            kind: SaPlayerStatEventKind::Shot,
            has_shot_ball: 1,
            shot_ball: rigid_body(
                SaVec3 {
                    x: 300.0,
                    y: 100.0,
                    z: 120.0,
                },
                SaVec3 {
                    x: 1000.0,
                    y: 500.0,
                    z: 100.0,
                },
            ),
            has_shot_player: 1,
            shot_player: rigid_body(
                SaVec3 {
                    x: 240.0,
                    y: 90.0,
                    z: 92.75,
                },
                SaVec3 {
                    x: 800.0,
                    y: 300.0,
                    z: 0.0,
                },
            ),
        }];
        let demolishes = [SaDemolishEvent {
            attacker_index: 0,
            victim_index: 1,
            attacker_velocity: SaVec3 {
                x: 2300.0,
                y: 0.0,
                z: 0.0,
            },
            victim_velocity: SaVec3::default(),
            victim_location: SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
            active_duration_seconds: 0.0,
        }];
        let mut frame = live_frame(
            1,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
                SaVec3::default(),
            ),
            &players,
        );
        frame.touches = touches.as_ptr();
        frame.touch_count = touches.len();
        frame.dodge_refreshes = dodge_refreshes.as_ptr();
        frame.dodge_refresh_count = dodge_refreshes.len();
        frame.boost_pad_events = boost_pad_events.as_ptr();
        frame.boost_pad_event_count = boost_pad_events.len();
        frame.goals = goals.as_ptr();
        frame.goal_count = goals.len();
        frame.player_stat_events = player_stat_events.as_ptr();
        frame.player_stat_event_count = player_stat_events.len();
        frame.demolishes = demolishes.as_ptr();
        frame.demolish_count = demolishes.len();

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );

        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        let player_frame = engine_ref
            .graph
            .state::<PlayerFrameState>()
            .expect("full analysis graph should expose player frame state");
        assert_eq!(frame_events.touch_events.len(), 1);
        assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
        assert_eq!(frame_events.boost_pad_events.len(), 1);
        assert_eq!(frame_events.goal_events.len(), 1);
        assert_eq!(frame_events.player_stat_events.len(), 1);
        assert_eq!(frame_events.demo_events.len(), 1);
        assert_eq!(frame_events.active_demos.len(), 1);
        assert_eq!(frame_events.boost_pad_events[0].pad_id, "34");
        assert_eq!(frame_events.goal_events[0].team_zero_score, Some(1));
        assert_eq!(
            frame_events.player_stat_events[0]
                .shot
                .as_ref()
                .expect("shot metadata should be populated")
                .ball_position
                .x,
            300.0
        );
        assert_eq!(frame_events.demo_events[0].victim, RemoteId::SplitScreen(1));
        assert_eq!(
            frame_events.active_demos[0].victim,
            RemoteId::SplitScreen(1)
        );
        let mut drained_event_buffer = [SaMechanicEvent {
            kind: SaMechanicKind::Shot,
            player_index: 0,
            is_team_0: 0,
            frame_number: 0,
            time: 0.0,
            confidence: 0.0,
        }; 64];
        let drained_count = unsafe {
            subtr_actor_bakkesmod_drain_events(
                engine,
                drained_event_buffer.as_mut_ptr(),
                drained_event_buffer.len(),
            )
        };
        let drained_events = &drained_event_buffer[..drained_count];
        assert!(
            drained_events.iter().any(|event| {
                event.kind == SaMechanicKind::Shot
                    && event.player_index == 0
                    && event.frame_number == 1
            }),
            "explicit live player stat events should drain through the full graph"
        );
        assert!(
            drained_events.iter().any(|event| {
                event.kind == SaMechanicKind::Demo
                    && event.player_index == 0
                    && event.frame_number == 1
            }),
            "explicit live demolish events should drain attacker demo events through the full graph"
        );
        assert!(
            drained_events.iter().any(|event| {
                event.kind == SaMechanicKind::Death
                    && event.player_index == 1
                    && event.frame_number == 1
            }),
            "explicit live demolish events should drain victim death events through the full graph"
        );
        let mut goal_context_events = [SaGoalContextEvent {
            frame_number: 0,
            time: 0.0,
            scoring_team_is_team_0: 0,
            has_scorer: 0,
            scorer_index: 0,
            has_scoring_team_most_back_player: 0,
            scoring_team_most_back_player_index: 0,
            has_defending_team_most_back_player: 0,
            defending_team_most_back_player_index: 0,
            has_ball_position: 0,
            ball_position: SaVec3::default(),
            has_ball_air_time_before_goal: 0,
            ball_air_time_before_goal: 0.0,
            goal_buildup: SaGoalBuildupKind::Other,
        }; 4];
        let goal_context_count = unsafe {
            subtr_actor_bakkesmod_drain_goal_context_events(
                engine,
                goal_context_events.as_mut_ptr(),
                goal_context_events.len(),
            )
        };
        assert_eq!(goal_context_count, 1);
        assert_eq!(goal_context_events[0].frame_number, 1);
        assert_eq!(goal_context_events[0].scoring_team_is_team_0, 1);
        assert_eq!(goal_context_events[0].has_scorer, 1);
        assert_eq!(goal_context_events[0].scorer_index, 0);
        assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
        let finalized_count = unsafe {
            subtr_actor_bakkesmod_drain_events(
                engine,
                drained_event_buffer.as_mut_ptr(),
                drained_event_buffer.len(),
            )
        };
        let finalized_events = &drained_event_buffer[..finalized_count];
        assert!(
            finalized_events.iter().any(|event| {
                event.kind == SaMechanicKind::Goal
                    && event.player_index == 0
                    && event.frame_number == 1
            }),
            "explicit live goal events should drain finalized goal events through the full graph"
        );
        assert_eq!(player_frame.players[1].match_goals, Some(1));
        assert_eq!(player_frame.players[1].match_assists, Some(2));
        assert_eq!(player_frame.players[1].match_saves, Some(3));
        assert_eq!(player_frame.players[1].match_shots, Some(4));
        assert_eq!(player_frame.players[1].match_score, Some(101));
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn live_processor_view_exposes_sampled_jump_state() {
        let mut player = player_at_index(
            3,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 120.0,
            },
        );
        player.jump_active = 1;
        player.double_jump_active = 1;
        player.dodge_active = 1;
        let players = [player];
        let frame = live_frame(1, SaRigidBody::default(), &players);
        let view = SaLiveProcessorView::new(None, &frame, &players, FrameEventsState::default());
        let player_id = RemoteId::SplitScreen(3);

        assert_eq!(view.get_jump_active(&player_id).unwrap(), 1);
        assert_eq!(view.get_double_jump_active(&player_id).unwrap(), 1);
        assert_eq!(view.get_dodge_active(&player_id).unwrap(), 1);
    }

    #[test]
    fn live_processor_view_resolves_demo_car_actor_ids() {
        let players = [
            player_at_index(
                2,
                true,
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                5,
                false,
                SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
        ];
        let frame = live_frame(
            7,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        let frame_info = frame_info(&frame);
        let demo_events = explicit_demolish_events(
            &frame_info,
            &[SaDemolishEvent {
                attacker_index: 2,
                victim_index: 5,
                attacker_velocity: SaVec3 {
                    x: 2300.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: SaVec3::default(),
                victim_location: SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
                active_duration_seconds: 0.25,
            }],
        );
        let view = SaLiveProcessorView::new(
            None,
            &frame,
            &players,
            FrameEventsState {
                active_demos: vec![DemoEventSample {
                    attacker: RemoteId::SplitScreen(2),
                    victim: RemoteId::SplitScreen(5),
                }],
                demo_events,
                ..FrameEventsState::default()
            },
        );

        let active_demos = view.get_active_demos().unwrap();
        assert_eq!(active_demos.len(), 1);
        assert_eq!(
            view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
                .unwrap(),
            RemoteId::SplitScreen(2)
        );
        assert_eq!(
            view.get_player_id_from_car_id(&active_demos[0].victim_actor_id())
                .unwrap(),
            RemoteId::SplitScreen(5)
        );
        assert_eq!(active_demos[0].attacker_velocity().x, 2300.0);
    }

    #[test]
    fn live_frame_input_can_build_active_demos_from_processor_view() {
        let players = [
            player_at_index(
                0,
                true,
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                1,
                false,
                SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
        ];
        let frame = live_frame(
            7,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        let frame_info = frame_info(&frame);
        let demo_events = explicit_demolish_events(
            &frame_info,
            &[SaDemolishEvent {
                attacker_index: 0,
                victim_index: 1,
                attacker_velocity: SaVec3 {
                    x: 2300.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: SaVec3::default(),
                victim_location: SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
                active_duration_seconds: 0.25,
            }],
        );
        let view = SaLiveProcessorView::new(
            None,
            &frame,
            &players,
            FrameEventsState {
                demo_events,
                ..FrameEventsState::default()
            },
        );

        let input = FrameInput::timeline_with_live_play_state(
            &view,
            7,
            frame.time,
            frame.dt,
            LivePlayState {
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
            },
        );

        let frame_events = input.frame_events_state();
        assert_eq!(frame_events.active_demos.len(), 1);
        assert_eq!(
            frame_events.active_demos[0].attacker,
            RemoteId::SplitScreen(0)
        );
        assert_eq!(
            frame_events.active_demos[0].victim,
            RemoteId::SplitScreen(1)
        );
    }

    #[test]
    fn live_processor_view_frame_input_preserves_live_event_streams() {
        let players = [
            player_at_index(
                0,
                true,
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                1,
                false,
                SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
        ];
        let frame = live_frame(
            7,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        let frame_info = frame_info(&frame);
        let demo_events = explicit_demolish_events(
            &frame_info,
            &[SaDemolishEvent {
                attacker_index: 0,
                victim_index: 1,
                attacker_velocity: SaVec3 {
                    x: 2300.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: SaVec3::default(),
                victim_location: SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
                active_duration_seconds: 0.25,
            }],
        );
        let view = SaLiveProcessorView::new(
            None,
            &frame,
            &players,
            FrameEventsState {
                active_demos: vec![DemoEventSample {
                    attacker: RemoteId::SplitScreen(0),
                    victim: RemoteId::SplitScreen(1),
                }],
                demo_events,
                ..FrameEventsState::default()
            },
        );

        let live_play = LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        };
        let input = FrameInput::timeline_with_live_play_state(
            &view,
            7,
            frame.time,
            frame.dt,
            live_play.clone(),
        );

        let frame_events = input.frame_events_state();
        assert_eq!(frame_events.demo_events.len(), 1);
        assert_eq!(
            frame_events.demo_events[0].attacker,
            RemoteId::SplitScreen(0)
        );
        assert_eq!(frame_events.active_demos.len(), 1);
        assert_eq!(
            frame_events.active_demos[0].victim,
            RemoteId::SplitScreen(1)
        );
        let player_frame = input.player_frame_state();
        assert_eq!(player_frame.players.len(), 2);
        assert_eq!(player_frame.players[1].match_score, Some(101));
        assert_eq!(input.live_play_state(), Some(live_play));
    }

    #[test]
    fn live_demolish_events_keep_active_demo_state_until_expiration() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [
            player_at_index(
                0,
                true,
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
            player_at_index(
                1,
                false,
                SaVec3 {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            ),
        ];
        let demolishes = [SaDemolishEvent {
            attacker_index: 0,
            victim_index: 1,
            attacker_velocity: SaVec3 {
                x: 2300.0,
                y: 0.0,
                z: 0.0,
            },
            victim_velocity: SaVec3::default(),
            victim_location: SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
            active_duration_seconds: 0.25,
        }];

        let mut first = live_frame(
            1,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        first.demolishes = demolishes.as_ptr();
        first.demolish_count = demolishes.len();
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
            0
        );

        let second = live_frame(
            2,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
            0
        );
        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        assert_eq!(frame_events.demo_events.len(), 0);
        assert_eq!(frame_events.active_demos.len(), 1);
        assert_eq!(
            frame_events.active_demos[0].victim,
            RemoteId::SplitScreen(1)
        );
        let demo = engine_ref
            .graph
            .state::<DemoCalculator>()
            .expect("full analysis graph should expose demo calculator state");
        assert_eq!(demo.timeline().len(), 2);

        let fourth = live_frame(
            4,
            rigid_body(SaVec3::default(), SaVec3::default()),
            &players,
        );
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &fourth) },
            0
        );
        let frame_events = engine_ref
            .graph
            .state::<FrameEventsState>()
            .expect("full analysis graph should expose frame events state");
        assert!(frame_events.active_demos.is_empty());
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn process_frame_feeds_explicit_live_touch_events_to_touch_state() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let players = [player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        )];
        let touches = [SaTouchEvent {
            player_index: 0,
            has_player: 1,
            is_team_0: 1,
            closest_approach_distance: 12.0,
            has_closest_approach_distance: 1,
        }];
        let mut frame = live_frame(
            1,
            rigid_body(
                SaVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 92.75,
                },
                SaVec3::default(),
            ),
            &players,
        );
        frame.touches = touches.as_ptr();
        frame.touch_count = touches.len();

        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );

        let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
        let touch_state = engine_ref
            .graph
            .state::<TouchState>()
            .expect("full analysis graph should expose touch state");
        assert_eq!(touch_state.touch_events.len(), 1);
        assert_eq!(
            touch_state.touch_events[0].player,
            Some(RemoteId::SplitScreen(0))
        );
        assert_eq!(
            touch_state.touch_events[0].closest_approach_distance,
            Some(12.0)
        );
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn rejects_null_explicit_event_pointer_when_count_is_nonzero() {
        let engine = subtr_actor_bakkesmod_engine_create();
        let mut frame = SaLiveFrame {
            frame_number: 1,
            live_play: 1,
            ..SaLiveFrame::default()
        };
        frame.touch_count = 1;

        let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

        assert_eq!(status, -1);
        unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    }

    #[test]
    fn emits_late_inserted_sorted_timeline_mechanics() {
        let mut pending_events = Vec::new();
        let mut emitted_mechanic_ids = HashSet::new();

        push_mechanic_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &[
                normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
                normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
            ],
        );
        assert_eq!(pending_events.len(), 2);

        pending_events.clear();
        push_mechanic_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &[
                normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
                normalized_mechanic("center:15:30:0", "center", 15, 1.5),
                normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
            ],
        );

        assert_eq!(pending_events.len(), 1);
        assert_eq!(pending_events[0].kind, SaMechanicKind::Center);
        assert_eq!(pending_events[0].frame_number, 15);
        assert_eq!(pending_events[0].time, 1.5);
    }

    #[test]
    fn drains_player_owned_events_from_timeline_events() {
        let mut pending_events = Vec::new();
        let mut emitted_mechanic_ids = HashSet::new();
        let mut pending_team_events = Vec::new();
        let mut emitted_team_event_ids = HashSet::new();
        let mut pending_goal_context_events = Vec::new();
        let mut emitted_goal_context_ids = HashSet::new();
        let timeline_events = ReplayStatsTimelineEvents {
            timeline: vec![
                TimelineEvent {
                    time: 1.05,
                    frame: Some(10),
                    kind: TimelineEventKind::Goal,
                    player_id: Some(RemoteId::SplitScreen(0)),
                    is_team_0: Some(true),
                },
                TimelineEvent {
                    time: 1.06,
                    frame: Some(10),
                    kind: TimelineEventKind::Shot,
                    player_id: Some(RemoteId::SplitScreen(0)),
                    is_team_0: Some(true),
                },
                TimelineEvent {
                    time: 1.07,
                    frame: Some(10),
                    kind: TimelineEventKind::Save,
                    player_id: Some(RemoteId::SplitScreen(1)),
                    is_team_0: Some(false),
                },
                TimelineEvent {
                    time: 1.08,
                    frame: Some(10),
                    kind: TimelineEventKind::Assist,
                    player_id: Some(RemoteId::SplitScreen(0)),
                    is_team_0: Some(true),
                },
                TimelineEvent {
                    time: 1.35,
                    frame: Some(13),
                    kind: TimelineEventKind::Kill,
                    player_id: Some(RemoteId::SplitScreen(0)),
                    is_team_0: Some(true),
                },
                TimelineEvent {
                    time: 1.35,
                    frame: Some(13),
                    kind: TimelineEventKind::Death,
                    player_id: Some(RemoteId::SplitScreen(1)),
                    is_team_0: Some(false),
                },
            ],
            goal_context: vec![goal_context_event(10, 1.09)],
            mechanics: vec![normalized_mechanic(
                "speed_flip:15:0",
                "speed_flip",
                15,
                1.5,
            )],
            backboard: vec![backboard_event(11, 1.1)],
            whiff: vec![whiff_event(12, 1.2, 0)],
            boost_pickups: vec![boost_pickup_event(125, 1.25)],
            bump: vec![bump_event(13, 1.3, 0.42)],
            fifty_fifty: vec![fifty_fifty_event(9, 14, 1.4)],
            goal_tags: vec![
                goal_tag_event(GoalTagKind::FlickGoal, Some(RemoteId::SplitScreen(1))),
                goal_tag_event(GoalTagKind::AerialGoal, None),
            ],
            rush: vec![rush_event(8, 16, 1.6, true)],
            ..ReplayStatsTimelineEvents::default()
        };

        push_drainable_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &mut pending_team_events,
            &mut emitted_team_event_ids,
            &mut pending_goal_context_events,
            &mut emitted_goal_context_ids,
            &timeline_events,
        );

        assert_eq!(pending_events.len(), 13);
        assert_eq!(pending_events[0].kind, SaMechanicKind::Goal);
        assert_eq!(pending_events[0].frame_number, 10);
        assert_eq!(pending_events[0].player_index, 0);
        assert_eq!(pending_events[1].kind, SaMechanicKind::Shot);
        assert_eq!(pending_events[1].frame_number, 10);
        assert_eq!(pending_events[1].player_index, 0);
        assert_eq!(pending_events[2].kind, SaMechanicKind::Save);
        assert_eq!(pending_events[2].frame_number, 10);
        assert_eq!(pending_events[2].player_index, 1);
        assert_eq!(pending_events[3].kind, SaMechanicKind::Assist);
        assert_eq!(pending_events[3].frame_number, 10);
        assert_eq!(pending_events[3].player_index, 0);
        assert_eq!(pending_events[4].kind, SaMechanicKind::Backboard);
        assert_eq!(pending_events[4].frame_number, 11);
        assert_eq!(pending_events[4].player_index, 0);
        assert_eq!(pending_events[5].kind, SaMechanicKind::Whiff);
        assert_eq!(pending_events[5].frame_number, 12);
        assert_eq!(pending_events[5].player_index, 0);
        assert_eq!(pending_events[6].kind, SaMechanicKind::BoostPickup);
        assert_eq!(pending_events[6].frame_number, 125);
        assert_eq!(pending_events[6].player_index, 0);
        assert_eq!(pending_events[7].kind, SaMechanicKind::Bump);
        assert_eq!(pending_events[7].frame_number, 13);
        assert_eq!(pending_events[7].player_index, 0);
        assert_eq!(pending_events[7].confidence, 0.42);
        assert_eq!(pending_events[8].kind, SaMechanicKind::Demo);
        assert_eq!(pending_events[8].time, 1.35);
        assert_eq!(pending_events[8].frame_number, 13);
        assert_eq!(pending_events[8].player_index, 0);
        assert_eq!(pending_events[9].kind, SaMechanicKind::Death);
        assert_eq!(pending_events[9].time, 1.35);
        assert_eq!(pending_events[9].frame_number, 13);
        assert_eq!(pending_events[9].player_index, 1);
        assert_eq!(pending_events[9].is_team_0, 0);
        assert_eq!(pending_events[10].kind, SaMechanicKind::FlickGoal);
        assert_eq!(pending_events[10].time, 1.36);
        assert_eq!(pending_events[10].frame_number, 13);
        assert_eq!(pending_events[10].player_index, 1);
        assert_eq!(pending_events[10].is_team_0, 0);
        assert_eq!(pending_events[10].confidence, 0.72);
        assert_eq!(pending_events[11].kind, SaMechanicKind::FiftyFifty);
        assert_eq!(pending_events[11].frame_number, 14);
        assert_eq!(pending_events[11].player_index, 1);
        assert_eq!(pending_events[11].is_team_0, 0);
        assert_eq!(pending_events[12].kind, SaMechanicKind::SpeedFlip);
        assert_eq!(pending_team_events.len(), 1);
        assert_eq!(pending_team_events[0].kind, SaTeamEventKind::Rush);
        assert_eq!(pending_team_events[0].is_team_0, 1);
        assert_eq!(pending_team_events[0].start_frame, 8);
        assert_eq!(pending_team_events[0].end_frame, 16);
        assert_eq!(pending_team_events[0].start_time, 1.0);
        assert_eq!(pending_team_events[0].end_time, 1.6);
        assert_eq!(pending_team_events[0].attackers, 3);
        assert_eq!(pending_team_events[0].defenders, 2);
        assert_eq!(pending_goal_context_events.len(), 1);
        assert_eq!(pending_goal_context_events[0].frame_number, 10);
        assert_eq!(pending_goal_context_events[0].time, 1.09);
        assert_eq!(pending_goal_context_events[0].scoring_team_is_team_0, 0);
        assert_eq!(pending_goal_context_events[0].has_scorer, 1);
        assert_eq!(pending_goal_context_events[0].scorer_index, 1);
        assert_eq!(
            pending_goal_context_events[0].has_defending_team_most_back_player,
            1
        );
        assert_eq!(
            pending_goal_context_events[0].defending_team_most_back_player_index,
            0
        );
        assert_eq!(pending_goal_context_events[0].has_ball_position, 1);
        assert_eq!(pending_goal_context_events[0].ball_position.x, 1.0);
        assert_eq!(
            pending_goal_context_events[0].has_ball_air_time_before_goal,
            1
        );
        assert_eq!(
            pending_goal_context_events[0].goal_buildup,
            SaGoalBuildupKind::CounterAttack
        );

        pending_events.clear();
        pending_team_events.clear();
        pending_goal_context_events.clear();
        push_drainable_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &mut pending_team_events,
            &mut emitted_team_event_ids,
            &mut pending_goal_context_events,
            &mut emitted_goal_context_ids,
            &timeline_events,
        );
        assert!(pending_events.is_empty());
        assert!(pending_team_events.is_empty());
        assert!(pending_goal_context_events.is_empty());
    }

    #[test]
    fn maps_normalized_timeline_mechanic_kinds_to_abi_kinds() {
        let expected_shared_graph_kinds = HashSet::from([
            "air_dribble",
            "ball_carry",
            "ceiling_shot",
            "center",
            "double_tap",
            "flick",
            "flip_reset",
            "half_flip",
            "half_volley",
            "musty_flick",
            "one_timer",
            "pass",
            "speed_flip",
            "wall_aerial",
            "wall_aerial_shot",
            "wavedash",
        ]);
        let shared_graph_kinds = STATS_TIMELINE_MECHANIC_KINDS
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        assert_eq!(
            shared_graph_kinds, expected_shared_graph_kinds,
            "shared stats timeline mechanic kind set changed; update ABI mapping expectations"
        );
        for &kind in STATS_TIMELINE_MECHANIC_KINDS {
            assert!(
                mechanic_kind(kind).is_some(),
                "BakkesMod ABI mapping must cover shared stats timeline mechanic kind: {kind}"
            );
        }

        assert_eq!(
            mechanic_kind("air_dribble"),
            Some(SaMechanicKind::AirDribble)
        );
        assert_eq!(mechanic_kind("ball_carry"), Some(SaMechanicKind::BallCarry));
        assert_eq!(
            mechanic_kind("ceiling_shot"),
            Some(SaMechanicKind::CeilingShot)
        );
        assert_eq!(mechanic_kind("center"), Some(SaMechanicKind::Center));
        assert_eq!(mechanic_kind("double_tap"), Some(SaMechanicKind::DoubleTap));
        assert_eq!(mechanic_kind("flick"), Some(SaMechanicKind::Flick));
        assert_eq!(mechanic_kind("flip_reset"), Some(SaMechanicKind::FlipReset));
        assert_eq!(mechanic_kind("half_flip"), Some(SaMechanicKind::HalfFlip));
        assert_eq!(
            mechanic_kind("half_volley"),
            Some(SaMechanicKind::HalfVolley)
        );
        assert_eq!(
            mechanic_kind("musty_flick"),
            Some(SaMechanicKind::MustyFlick)
        );
        assert_eq!(mechanic_kind("one_timer"), Some(SaMechanicKind::OneTimer));
        assert_eq!(mechanic_kind("pass"), Some(SaMechanicKind::Pass));
        assert_eq!(mechanic_kind("speed_flip"), Some(SaMechanicKind::SpeedFlip));
        assert_eq!(
            mechanic_kind("wall_aerial"),
            Some(SaMechanicKind::WallAerial)
        );
        assert_eq!(
            mechanic_kind("wall_aerial_shot"),
            Some(SaMechanicKind::WallAerialShot)
        );
        assert_eq!(mechanic_kind("wavedash"), Some(SaMechanicKind::Wavedash));
        assert_eq!(mechanic_kind("unmapped"), None);
    }

    #[test]
    fn maps_timeline_event_kinds_to_abi_kinds() {
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Goal),
            SaMechanicKind::Goal
        );
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Shot),
            SaMechanicKind::Shot
        );
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Save),
            SaMechanicKind::Save
        );
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Assist),
            SaMechanicKind::Assist
        );
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Kill),
            SaMechanicKind::Demo
        );
        assert_eq!(
            timeline_event_kind(TimelineEventKind::Death),
            SaMechanicKind::Death
        );
    }

    #[test]
    fn maps_goal_tag_kinds_to_abi_kinds() {
        assert_eq!(
            goal_tag_kind(GoalTagKind::AerialGoal),
            SaMechanicKind::AerialGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::HighAerialGoal),
            SaMechanicKind::HighAerialGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::LongDistanceGoal),
            SaMechanicKind::LongDistanceGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::OwnHalfGoal),
            SaMechanicKind::OwnHalfGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::EmptyNetGoal),
            SaMechanicKind::EmptyNetGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::CounterAttackGoal),
            SaMechanicKind::CounterAttackGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::FlickGoal),
            SaMechanicKind::FlickGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::DoubleTapGoal),
            SaMechanicKind::DoubleTapGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::OneTimerGoal),
            SaMechanicKind::OneTimerGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::AirDribbleGoal),
            SaMechanicKind::AirDribbleGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::FlipResetGoal),
            SaMechanicKind::FlipResetGoal
        );
        assert_eq!(
            goal_tag_kind(GoalTagKind::HalfVolleyGoal),
            SaMechanicKind::HalfVolleyGoal
        );
    }
}
