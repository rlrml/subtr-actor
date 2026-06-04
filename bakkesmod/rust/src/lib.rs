#![allow(clippy::result_large_err)]
#![allow(clippy::missing_safety_doc)]

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
#[cfg(test)]
use subtr_actor::ReplayFrameInputBuilder;
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
    BoostPickupComparisonEvent, BumpEvent, CorePlayerScoreboardEvent, DemoEventSample,
    DemolishAttribute, DemolishInfo, DodgeRefreshedEvent, FiftyFiftyEvent, FrameEventsState,
    FrameInfo, FrameInput, GameplayPhase, GameplayState, GoalBuildupKind, GoalContextEvent,
    GoalEvent, GoalTagEvent, GoalTagKind, LivePlayState, PlayerFrameState, PlayerId, PlayerInfo,
    PlayerSample, PlayerStatEvent, PlayerStatEventKind, ProcessorView, ReplayMeta,
    ReplayStatsFrame, ReplayStatsTimeline, ReplayStatsTimelineEvents, RushEvent, ShotEventMetadata,
    StatsEventTiming, StatsTimelineCollector, StatsTimelineEventCollector, StatsTimelineTagEvent,
    SubtrActorError, SubtrActorErrorVariant, SubtrActorResult, TimelineEvent, TimelineEventKind,
    TouchEvent, TouchStateCalculator, WhiffEvent,
};

mod abi;
mod ffi;
mod ffi_graph_output;
mod graph_output;
mod live_events;
mod live_processor;
mod timeline_drain;

pub use abi::*;
pub use ffi::*;
pub use ffi_graph_output::*;
pub(crate) use graph_output::*;
pub(crate) use live_events::*;
pub(crate) use live_processor::*;
pub(crate) use timeline_drain::*;

pub struct SaEngine {
    graph: AnalysisGraph,
    live_events: SaLiveEventGenerator,
    live_event_history: SaLiveEventHistory,
    live_replay_meta_initialized: bool,
    live_graph_finished: bool,
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
    "core_player_goal_context",
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
            live_graph_finished: false,
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

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
