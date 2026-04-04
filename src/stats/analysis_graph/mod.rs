#![allow(dead_code)]

use std::collections::HashSet;

use crate::Collector;
use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

pub mod graph;
pub use graph::{
    AnalysisDependency, AnalysisGraph, AnalysisNode, AnalysisNodeDyn, AnalysisStateContext,
    AnalysisStateRef,
};

#[macro_use]
mod node_macros;

mod backboard;
mod backboard_bounce;
mod ball_carry;
mod ball_frame_state;
mod boost;
mod ceiling_shot;
mod collector;
mod demo;
mod dodge_reset;
mod double_tap;
mod fifty_fifty;
mod fifty_fifty_state;
mod frame_events_state;
mod frame_info;
mod gameplay_state;
mod live_play;
mod match_stats;
mod movement;
mod musty_flick;
mod nodes;
mod player_frame_state;
mod player_vertical_state;
mod positioning;
mod possession;
mod possession_state;
mod powerslide;
mod pressure;
mod rush;
mod settings;
mod speed_flip;
mod stats_timeline_events;
mod stats_timeline_frame;
mod touch;
mod touch_state;

use crate::stats::calculators::FrameInput;

#[allow(unused_imports)]
pub use backboard::BackboardNode;
#[allow(unused_imports)]
pub use backboard_bounce::BackboardBounceStateNode;
#[allow(unused_imports)]
pub use ball_carry::BallCarryNode;
#[allow(unused_imports)]
pub use ball_frame_state::BallFrameStateNode;
#[allow(unused_imports)]
pub use boost::BoostNode;
#[allow(unused_imports)]
pub use ceiling_shot::CeilingShotNode;
#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use demo::DemoNode;
#[allow(unused_imports)]
pub use dodge_reset::DodgeResetNode;
#[allow(unused_imports)]
pub use double_tap::DoubleTapNode;
#[allow(unused_imports)]
pub use fifty_fifty::FiftyFiftyNode;
#[allow(unused_imports)]
pub use fifty_fifty_state::FiftyFiftyStateNode;
#[allow(unused_imports)]
pub use frame_events_state::FrameEventsStateNode;
#[allow(unused_imports)]
pub use frame_info::FrameInfoNode;
#[allow(unused_imports)]
pub use gameplay_state::GameplayStateNode;
#[allow(unused_imports)]
pub use live_play::LivePlayNode;
#[allow(unused_imports)]
pub use match_stats::MatchStatsNode;
#[allow(unused_imports)]
pub use movement::MovementNode;
#[allow(unused_imports)]
pub use musty_flick::MustyFlickNode;
#[allow(unused_imports)]
pub use player_frame_state::PlayerFrameStateNode;
#[allow(unused_imports)]
pub use player_vertical_state::PlayerVerticalStateNode;
#[allow(unused_imports)]
pub use positioning::PositioningNode;
#[allow(unused_imports)]
pub use possession::PossessionNode;
#[allow(unused_imports)]
pub use possession_state::PossessionStateNode;
#[allow(unused_imports)]
pub use powerslide::PowerslideNode;
#[allow(unused_imports)]
pub use pressure::PressureNode;
#[allow(unused_imports)]
pub use rush::RushNode;
#[allow(unused_imports)]
pub use settings::SettingsNode;
#[allow(unused_imports)]
pub use speed_flip::SpeedFlipNode;
#[allow(unused_imports)]
pub use stats_timeline_events::{StatsTimelineEventsNode, StatsTimelineEventsState};
#[allow(unused_imports)]
pub use stats_timeline_frame::{StatsTimelineFrameNode, StatsTimelineFrameState};
#[allow(unused_imports)]
pub use touch::TouchNode;
#[allow(unused_imports)]
pub use touch_state::TouchStateNode;

pub(crate) fn boxed_analysis_node_by_name(
    name: &str,
) -> Option<Box<dyn AnalysisNodeDyn>> {
    match name {
        "core" => Some(match_stats::boxed_default()),
        "backboard" => Some(backboard::boxed_default()),
        "ceiling_shot" => Some(ceiling_shot::boxed_default()),
        "double_tap" => Some(double_tap::boxed_default()),
        "fifty_fifty" => Some(fifty_fifty::boxed_default()),
        "possession" => Some(possession::boxed_default()),
        "pressure" => Some(pressure::boxed_default()),
        "rush" => Some(rush::boxed_default()),
        "touch" => Some(touch::boxed_default()),
        "speed_flip" => Some(speed_flip::boxed_default()),
        "musty_flick" => Some(musty_flick::boxed_default()),
        "dodge_reset" => Some(dodge_reset::boxed_default()),
        "ball_carry" => Some(ball_carry::boxed_default()),
        "boost" => Some(boost::boxed_default()),
        "movement" => Some(movement::boxed_default()),
        "positioning" => Some(positioning::boxed_default()),
        "powerslide" => Some(powerslide::boxed_default()),
        "demo" => Some(demo::boxed_default()),
        _ => None,
    }
}

pub fn graph_with_builtin_analysis_nodes<I, S>(
    names: I,
) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(live_play::boxed_default());
    let mut seen = HashSet::new();
    for name in names {
        let name = name.as_ref();
        if !seen.insert(name.to_owned()) {
            continue;
        }
        graph.push_boxed_node(boxed_analysis_node_by_name(name).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                name.to_owned(),
            ))
        })?);
    }
    Ok(graph)
}

pub fn collect_analysis_graph_for_replay(
    replay: &boxcars::Replay,
    graph: AnalysisGraph,
) -> SubtrActorResult<AnalysisGraph> {
    let collector = collector::AnalysisNodeCollector::new(graph).process_replay(replay)?;
    Ok(collector.into_graph())
}

pub fn collect_builtin_analysis_graph_for_replay<I, S>(
    replay: &boxcars::Replay,
    names: I,
) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    collect_analysis_graph_for_replay(replay, graph_with_builtin_analysis_nodes(names)?)
}

pub fn all_analysis_nodes() -> Vec<Box<dyn AnalysisNodeDyn>> {
    vec![
        backboard::boxed_default(),
        ball_carry::boxed_default(),
        boost::boxed_default(),
        ceiling_shot::boxed_default(),
        demo::boxed_default(),
        dodge_reset::boxed_default(),
        double_tap::boxed_default(),
        fifty_fifty::boxed_default(),
        match_stats::boxed_default(),
        movement::boxed_default(),
        musty_flick::boxed_default(),
        positioning::boxed_default(),
        possession::boxed_default(),
        powerslide::boxed_default(),
        pressure::boxed_default(),
        rush::boxed_default(),
        settings::boxed_default(),
        speed_flip::boxed_default(),
        touch::boxed_default(),
    ]
}

pub fn graph_with_all_analysis_nodes() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(live_play::boxed_default());
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
