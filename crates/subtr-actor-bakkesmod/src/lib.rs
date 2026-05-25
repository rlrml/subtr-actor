use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{
    stats::analysis_graph::{
        graph_with_all_analysis_nodes, AnalysisGraph, StatsTimelineEventsNode,
        StatsTimelineEventsState, StatsTimelineFrameNode, StatsTimelineFrameState,
    },
    BackboardBounceEvent, BallFrameState, BallSample, BoostPadEvent, BoostPadEventKind,
    BoostPickupComparisonEvent, BumpEvent, DemoEventSample, DemolishInfo, DodgeRefreshedEvent,
    FrameEventsState, FrameInfo, FrameInput, GameplayPhase, GameplayState, GoalEvent,
    LivePlayState, MechanicEvent, MechanicTiming, PlayerFrameState, PlayerInfo, PlayerSample,
    PlayerStatEvent, PlayerStatEventKind, ReplayMeta, ReplayStatsTimelineEvents, ShotEventMetadata,
    TimelineEvent, TimelineEventKind, TouchEvent, TouchState, TouchStateCalculator, WhiffEvent,
    WhiffEventKind,
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

pub struct SaEngine {
    graph: AnalysisGraph,
    live_events: SaLiveEventGenerator,
    live_replay_meta_initialized: bool,
    live_replay_meta_signature: Vec<(RemoteId, bool, Option<String>)>,
    emitted_mechanic_ids: HashSet<String>,
    events_json: Vec<u8>,
    frame_json: Vec<u8>,
    pending_events: Vec<SaMechanicEvent>,
}

impl Default for SaEngine {
    fn default() -> Self {
        let mut graph = graph_with_all_analysis_nodes();
        graph.push_boxed_node(Box::new(StatsTimelineFrameNode::new()));
        graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
        Self {
            graph,
            live_events: SaLiveEventGenerator::default(),
            live_replay_meta_initialized: false,
            live_replay_meta_signature: Vec::new(),
            emitted_mechanic_ids: HashSet::new(),
            events_json: Vec::new(),
            frame_json: Vec::new(),
            pending_events: Vec::new(),
        }
    }
}

#[derive(Default)]
struct SaLiveEventGenerator {
    touch_state: TouchStateCalculator,
    live_play_tracker: subtr_actor::LivePlayTracker,
    dodge_refresh_counters: Vec<(RemoteId, i32)>,
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

struct SaFrameEventSlices<'a> {
    touches: &'a [SaTouchEvent],
    dodge_refreshes: &'a [SaDodgeRefreshedEvent],
    boost_pad_events: &'a [SaBoostPadEvent],
    goals: &'a [SaGoalEvent],
    player_stat_events: &'a [SaPlayerStatEvent],
    demolishes: &'a [SaDemolishEvent],
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

fn explicit_active_demo_events(events: &[SaDemolishEvent]) -> Vec<DemoEventSample> {
    events
        .iter()
        .map(|event| DemoEventSample {
            attacker: player_id(event.attacker_index),
            victim: player_id(event.victim_index),
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
        let active_demos = explicit_active_demo_events(explicit_events.demolishes);
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
    let (frame_events, live_play) = engine.live_events.frame_events(
        &frame_info,
        &ball,
        &players,
        &gameplay,
        explicit_live_play_state(frame),
        explicit_events,
    );
    FrameInput::from_parts_with_live_play_state(
        frame_info,
        gameplay,
        ball,
        players,
        frame_events,
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

fn push_demo_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    timeline: &[TimelineEvent],
) {
    for (index, event) in timeline.iter().enumerate() {
        if event.kind != TimelineEventKind::Kill {
            continue;
        }
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!("demo:{:.3}:{}:{index}", event.time, player_index(player_id)),
                kind: SaMechanicKind::Demo,
                player_id: player_id.clone(),
                is_team_0,
                frame_number: 0,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

fn push_drainable_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
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
    push_demo_events_from_timeline(pending_events, emitted_mechanic_ids, &events.timeline);
    pending_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
}

fn refresh_timeline_graph_views(engine: &mut SaEngine) {
    let Some(timeline_events) = engine.graph.state::<StatsTimelineEventsState>() else {
        engine.events_json.clear();
        engine.frame_json.clear();
        return;
    };
    push_drainable_events_from_timeline(
        &mut engine.pending_events,
        &mut engine.emitted_mechanic_ids,
        &timeline_events.events,
    );
    engine.events_json = serde_json::to_vec(&timeline_events.events).unwrap_or_default();

    engine.frame_json = engine
        .graph
        .state::<StatsTimelineFrameState>()
        .and_then(|state| state.frame.as_ref())
        .and_then(|frame| serde_json::to_vec(frame).ok())
        .unwrap_or_default();
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
    let frame_input = frame_input(engine, frame, players, &explicit_events);
    if sync_live_replay_meta(engine, players).is_err() {
        return -2;
    }
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
    use subtr_actor::{
        BoostPickupActivity, BoostPickupComparison, BoostPickupFieldHalf, BoostPickupPadType,
    };

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
        }; 4];

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

        assert_eq!(
            unsafe {
                subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len())
            },
            0
        );
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
        assert_eq!(player_frame.players[1].match_goals, Some(1));
        assert_eq!(player_frame.players[1].match_assists, Some(2));
        assert_eq!(player_frame.players[1].match_saves, Some(3));
        assert_eq!(player_frame.players[1].match_shots, Some(4));
        assert_eq!(player_frame.players[1].match_score, Some(101));
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
        let timeline_events = ReplayStatsTimelineEvents {
            timeline: vec![
                TimelineEvent {
                    time: 1.35,
                    kind: TimelineEventKind::Kill,
                    player_id: Some(RemoteId::SplitScreen(0)),
                    is_team_0: Some(true),
                },
                TimelineEvent {
                    time: 1.35,
                    kind: TimelineEventKind::Death,
                    player_id: Some(RemoteId::SplitScreen(1)),
                    is_team_0: Some(false),
                },
            ],
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
            ..ReplayStatsTimelineEvents::default()
        };

        push_drainable_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &timeline_events,
        );

        assert_eq!(pending_events.len(), 6);
        assert_eq!(pending_events[0].kind, SaMechanicKind::Backboard);
        assert_eq!(pending_events[0].frame_number, 11);
        assert_eq!(pending_events[0].player_index, 0);
        assert_eq!(pending_events[1].kind, SaMechanicKind::Whiff);
        assert_eq!(pending_events[1].frame_number, 12);
        assert_eq!(pending_events[1].player_index, 0);
        assert_eq!(pending_events[2].kind, SaMechanicKind::BoostPickup);
        assert_eq!(pending_events[2].frame_number, 125);
        assert_eq!(pending_events[2].player_index, 0);
        assert_eq!(pending_events[3].kind, SaMechanicKind::Bump);
        assert_eq!(pending_events[3].frame_number, 13);
        assert_eq!(pending_events[3].player_index, 0);
        assert_eq!(pending_events[3].confidence, 0.42);
        assert_eq!(pending_events[4].kind, SaMechanicKind::Demo);
        assert_eq!(pending_events[4].time, 1.35);
        assert_eq!(pending_events[4].player_index, 0);
        assert_eq!(pending_events[5].kind, SaMechanicKind::SpeedFlip);

        pending_events.clear();
        push_drainable_events_from_timeline(
            &mut pending_events,
            &mut emitted_mechanic_ids,
            &timeline_events,
        );
        assert!(pending_events.is_empty());
    }

    #[test]
    fn maps_normalized_timeline_mechanic_kinds_to_abi_kinds() {
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
}
