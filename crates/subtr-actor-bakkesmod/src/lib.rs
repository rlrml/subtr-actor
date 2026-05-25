use std::ptr;
use std::slice;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{
    stats::analysis_graph::{
        graph_with_all_analysis_nodes, AnalysisGraph, StatsTimelineEventsNode,
        StatsTimelineEventsState,
    },
    BallFrameState, BallSample, BoostPadEvent, BoostPadEventKind, DemoEventSample, DemolishInfo,
    DodgeRefreshedEvent, FrameEventsState, FrameInfo, FrameInput, GameplayPhase, GameplayState,
    GoalEvent, LivePlayState, MechanicEvent, MechanicTiming, PlayerFrameState, PlayerSample,
    PlayerStatEvent, PlayerStatEventKind, ShotEventMetadata, TouchEvent, TouchStateCalculator,
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
    last_mechanic_count: usize,
    pending_events: Vec<SaMechanicEvent>,
}

impl Default for SaEngine {
    fn default() -> Self {
        let mut graph = graph_with_all_analysis_nodes();
        graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
        Self {
            graph,
            live_events: SaLiveEventGenerator::default(),
            last_mechanic_count: 0,
            pending_events: Vec::new(),
        }
    }
}

#[derive(Default)]
struct SaLiveEventGenerator {
    touch_state: TouchStateCalculator,
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

fn live_play_state(frame: &SaLiveFrame) -> LivePlayState {
    let is_live_play = frame.live_play != 0;
    LivePlayState {
        gameplay_phase: if is_live_play {
            GameplayPhase::ActivePlay
        } else {
            GameplayPhase::Unknown
        },
        is_live_play,
    }
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
        live_play: &LivePlayState,
        explicit_events: &SaFrameEventSlices<'_>,
    ) -> FrameEventsState {
        let empty_events = FrameEventsState::default();
        let touch_state = self
            .touch_state
            .update(frame, ball, players, &empty_events, live_play);
        let mut touch_events = explicit_touch_events(frame, explicit_events.touches);
        if touch_events.is_empty() {
            touch_events.extend(touch_state.touch_events);
        }
        let dodge_refreshed_events = infer_dodge_refreshed_events(
            frame,
            ball,
            players,
            &touch_events,
            &mut self.dodge_refresh_counters,
        );
        let mut dodge_refreshed_events =
            explicit_dodge_refreshed_events(frame, explicit_events.dodge_refreshes)
                .into_iter()
                .chain(dodge_refreshed_events)
                .collect::<Vec<_>>();
        dodge_refreshed_events.sort_by_key(|event| event.counter_value);

        let demo_events = explicit_demolish_events(frame, explicit_events.demolishes);
        let active_demos = explicit_active_demo_events(explicit_events.demolishes);

        FrameEventsState {
            active_demos,
            demo_events,
            boost_pad_events: explicit_boost_pad_events(frame, explicit_events.boost_pad_events),
            touch_events,
            dodge_refreshed_events,
            player_stat_events: explicit_player_stat_events(
                frame,
                explicit_events.player_stat_events,
            ),
            goal_events: explicit_goal_events(frame, explicit_events.goals),
            ..FrameEventsState::default()
        }
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
    let live_play = live_play_state(frame);
    let frame_events =
        engine
            .live_events
            .frame_events(&frame_info, &ball, &players, &live_play, explicit_events);
    FrameInput::from_parts(
        frame_info,
        gameplay_state(frame, sampled_players),
        ball,
        players,
        frame_events,
    )
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

fn push_new_events(engine: &mut SaEngine) {
    let Some(timeline_events) = engine.graph.state::<StatsTimelineEventsState>() else {
        return;
    };
    let mechanics = &timeline_events.events.mechanics;
    for event in &mechanics[engine.last_mechanic_count..] {
        let Some(kind) = mechanic_kind(&event.kind) else {
            continue;
        };
        let (frame_number, time) = mechanic_start(event);
        engine.pending_events.push(SaMechanicEvent {
            kind,
            player_index: player_index(&event.player_id),
            is_team_0: event.is_team_0 as u8,
            frame_number: frame_number as u64,
            time,
            confidence: 1.0,
        });
    }
    engine.last_mechanic_count = mechanics.len();
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
    let Ok(explicit_events) = frame_event_slices(frame) else {
        return -1;
    };
    let frame_input = frame_input(engine, frame, players, &explicit_events);
    if engine.graph.evaluate_with_state(&frame_input).is_err() {
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
