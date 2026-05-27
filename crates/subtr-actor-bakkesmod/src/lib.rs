#![allow(clippy::result_large_err)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use boxcars::{ParserBuilder, Quaternion, RemoteId, RigidBody, Vector3f};
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
    GoalEvent, GoalTagEvent, GoalTagKind, LivePlayState, MechanicEvent, MechanicTiming,
    PlayerFrameState, PlayerId, PlayerInfo, PlayerSample, PlayerStatEvent, PlayerStatEventKind,
    ProcessorBallView, ProcessorEventHistoryView, ProcessorFrameEventView, ProcessorGameView,
    ProcessorPlayerCoreView, ProcessorPlayerStatsView, ReplayMeta, ReplayStatsFrame,
    ReplayStatsTimeline, ReplayStatsTimelineEvents, RushEvent, ShotEventMetadata,
    StatsTimelineEventCollector, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult,
    TimelineEvent, TimelineEventKind, TouchEvent, TouchStateCalculator, WhiffEvent,
};

#[path = "abi.rs"]
mod abi;
pub use abi::*;

#[path = "engine.rs"]
mod engine;
pub use engine::{SaEngine, SaReplayAnnotations};
#[path = "engine_constants.rs"]
mod engine_constants;
use engine_constants::*;

#[path = "live.rs"]
mod live;
use live::*;

#[path = "timeline.rs"]
mod timeline;
use timeline::*;

#[path = "graph_output.rs"]
mod graph_output;
use graph_output::*;

#[path = "replay_annotations.rs"]
mod replay_annotations;
use replay_annotations::*;

#[path = "ffi_lifecycle.rs"]
mod ffi_lifecycle;
use ffi_lifecycle::*;
#[path = "ffi_process_frame.rs"]
mod ffi_process_frame;
use ffi_process_frame::*;
#[path = "ffi_pending.rs"]
mod ffi_pending;
use ffi_pending::*;
#[path = "ffi_live_json.rs"]
mod ffi_live_json;
use ffi_live_json::*;
#[path = "ffi_stats_json.rs"]
mod ffi_stats_json;
use ffi_stats_json::*;
#[path = "ffi_graph_json.rs"]
mod ffi_graph_json;
use ffi_graph_json::*;
#[path = "ffi_drain.rs"]
mod ffi_drain;
use ffi_drain::*;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
