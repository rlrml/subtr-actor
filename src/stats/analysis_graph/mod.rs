use std::collections::HashSet;

use serde::Serialize;

use crate::Collector;
use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

pub mod graph;
pub use graph::{
    AnalysisDependency, AnalysisGraph, AnalysisNode, AnalysisNodeDyn, AnalysisStateContext,
    AnalysisStateRef,
};

#[macro_use]
mod node_macros;

mod collector;
mod nodes;

use crate::stats::calculators::FrameInput;
#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use nodes::*;

pub const BUILTIN_ANALYSIS_NODE_NAMES: &[&str] = &[
    "core",
    "frame_info",
    "gameplay_state",
    "ball_frame_state",
    "player_frame_state",
    "frame_events_state",
    "live_play",
    "match_stats",
    "backboard",
    "backboard_bounce_state",
    "ceiling_shot",
    "center",
    "controlled_play",
    "continuous_ball_control",
    "double_tap",
    "fifty_fifty",
    "fifty_fifty_state",
    "kickoff",
    "possession",
    "possession_state",
    "ball_half",
    "territorial_pressure",
    "rotation",
    "rush",
    "touch",
    "touch_state",
    "wall_aerial",
    "wall_aerial_shot",
    "whiff",
    "wavedash",
    "dodge",
    "speed_flip",
    "half_flip",
    "half_volley",
    "flick",
    "aerial_goal",
    "high_aerial_goal",
    "long_distance_goal",
    "own_half_goal",
    "empty_net_goal",
    "counter_attack_goal",
    "sustained_pressure_goal",
    "kickoff_goal",
    "flick_goal",
    "ceiling_shot_goal",
    "double_tap_goal",
    "one_timer_goal",
    "passing_goal",
    "air_dribble_goal",
    "flip_reset_goal",
    "bump_goal",
    "demo_goal",
    "half_volley_goal",
    "musty_flick",
    "one_timer",
    "pass",
    "dodge_reset",
    "ball_carry",
    "air_dribble",
    "boost",
    "bump",
    "movement",
    "positioning",
    "powerslide",
    "player_vertical_state",
    "demo",
    "settings",
    "stats_projection",
    "stats_timeline_frame",
    "stats_timeline_events",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuiltinAnalysisNodeAlias {
    pub alias: &'static str,
    pub node_name: &'static str,
}

pub const BUILTIN_ANALYSIS_NODE_ALIASES: &[BuiltinAnalysisNodeAlias] = &[
    BuiltinAnalysisNodeAlias {
        alias: "core",
        node_name: "match_stats",
    },
    BuiltinAnalysisNodeAlias {
        alias: "air_dribble",
        node_name: "ball_carry",
    },
    BuiltinAnalysisNodeAlias {
        alias: "flip_impulse",
        node_name: "dodge",
    },
];

pub fn builtin_analysis_node_names() -> &'static [&'static str] {
    BUILTIN_ANALYSIS_NODE_NAMES
}

pub fn builtin_analysis_node_aliases() -> &'static [BuiltinAnalysisNodeAlias] {
    BUILTIN_ANALYSIS_NODE_ALIASES
}

pub(crate) fn canonical_builtin_analysis_node_name(name: &str) -> Option<&'static str> {
    builtin_analysis_node_aliases()
        .iter()
        .find_map(|alias| (alias.alias == name).then_some(alias.node_name))
        .or_else(|| {
            builtin_analysis_node_names()
                .iter()
                .copied()
                .find(|candidate| *candidate == name)
        })
}

pub(crate) fn boxed_analysis_node_by_name(name: &str) -> Option<Box<dyn AnalysisNodeDyn>> {
    match name {
        "core" => Some(nodes::match_stats::boxed_default()),
        "frame_info" => Some(nodes::frame_info::boxed_default()),
        "gameplay_state" => Some(nodes::gameplay_state::boxed_default()),
        "ball_frame_state" => Some(nodes::ball_frame_state::boxed_default()),
        "player_frame_state" => Some(nodes::player_frame_state::boxed_default()),
        "frame_events_state" => Some(nodes::frame_events_state::boxed_default()),
        "live_play" => Some(nodes::live_play::boxed_default()),
        "match_stats" => Some(nodes::match_stats::boxed_default()),
        "backboard" => Some(nodes::backboard::boxed_default()),
        "backboard_bounce_state" => Some(nodes::backboard_bounce::boxed_default()),
        "ceiling_shot" => Some(nodes::ceiling_shot::boxed_default()),
        "center" => Some(nodes::center::boxed_default()),
        "controlled_play" => Some(nodes::controlled_play::boxed_default()),
        "continuous_ball_control" => Some(nodes::continuous_ball_control::boxed_default()),
        "double_tap" => Some(nodes::double_tap::boxed_default()),
        "fifty_fifty" => Some(nodes::fifty_fifty::boxed_default()),
        "fifty_fifty_state" => Some(nodes::fifty_fifty_state::boxed_default()),
        "kickoff" => Some(nodes::kickoff::boxed_default()),
        "possession" => Some(nodes::possession::boxed_default()),
        "possession_state" => Some(nodes::possession_state::boxed_default()),
        "ball_half" => Some(nodes::ball_half::boxed_default()),
        "territorial_pressure" => Some(nodes::territorial_pressure::boxed_default()),
        "rotation" => Some(nodes::rotation::boxed_default()),
        "rush" => Some(nodes::rush::boxed_default()),
        "touch" => Some(nodes::touch::boxed_default()),
        "touch_state" => Some(nodes::touch_state::boxed_default()),
        "wall_aerial" => Some(nodes::wall_aerial::boxed_default()),
        "wall_aerial_shot" => Some(nodes::wall_aerial_shot::boxed_default()),
        "whiff" => Some(nodes::whiff::boxed_default()),
        "wavedash" => Some(nodes::wavedash::boxed_default()),
        "speed_flip" => Some(nodes::speed_flip::boxed_default()),
        "dodge" => Some(nodes::flip_impulse::boxed_default()),
        "half_flip" => Some(nodes::half_flip::boxed_default()),
        "half_volley" => Some(nodes::half_volley::boxed_default()),
        "flick" => Some(nodes::flick::boxed_default()),
        "aerial_goal" => Some(nodes::goal_tags::boxed_aerial_goal()),
        "high_aerial_goal" => Some(nodes::goal_tags::boxed_high_aerial_goal()),
        "long_distance_goal" => Some(nodes::goal_tags::boxed_long_distance_goal()),
        "own_half_goal" => Some(nodes::goal_tags::boxed_own_half_goal()),
        "empty_net_goal" => Some(nodes::goal_tags::boxed_empty_net_goal()),
        "counter_attack_goal" => Some(nodes::goal_tags::boxed_counter_attack_goal()),
        "sustained_pressure_goal" => Some(nodes::goal_tags::boxed_sustained_pressure_goal()),
        "kickoff_goal" => Some(nodes::goal_tags::boxed_kickoff_goal()),
        "flick_goal" => Some(nodes::goal_tags::boxed_flick_goal()),
        "ceiling_shot_goal" => Some(nodes::goal_tags::boxed_ceiling_shot_goal()),
        "double_tap_goal" => Some(nodes::goal_tags::boxed_double_tap_goal()),
        "one_timer_goal" => Some(nodes::goal_tags::boxed_one_timer_goal()),
        "passing_goal" => Some(nodes::goal_tags::boxed_passing_goal()),
        "air_dribble_goal" => Some(nodes::goal_tags::boxed_air_dribble_goal()),
        "flip_reset_goal" => Some(nodes::goal_tags::boxed_flip_reset_goal()),
        "bump_goal" => Some(nodes::goal_tags::boxed_bump_goal()),
        "demo_goal" => Some(nodes::goal_tags::boxed_demo_goal()),
        "half_volley_goal" => Some(nodes::goal_tags::boxed_half_volley_goal()),
        "musty_flick" => Some(nodes::musty_flick::boxed_default()),
        "one_timer" => Some(nodes::one_timer::boxed_default()),
        "pass" => Some(nodes::pass::boxed_default()),
        "dodge_reset" => Some(nodes::dodge_reset::boxed_default()),
        "ball_carry" => Some(nodes::ball_carry::boxed_default()),
        "boost" => Some(nodes::boost::boxed_default()),
        "bump" => Some(nodes::bump::boxed_default()),
        "movement" => Some(nodes::movement::boxed_default()),
        "positioning" => Some(nodes::positioning::boxed_default()),
        "powerslide" => Some(nodes::powerslide::boxed_default()),
        "player_vertical_state" => Some(nodes::player_vertical_state::boxed_default()),
        "demo" => Some(nodes::demo::boxed_default()),
        "settings" => Some(nodes::settings::boxed_default()),
        "stats_projection" => Some(nodes::stats_projection::boxed_default()),
        "stats_timeline_frame" => Some(nodes::stats_timeline_frame::boxed_default()),
        "stats_timeline_events" => Some(nodes::stats_timeline_events::boxed_default()),
        _ => None,
    }
}

pub fn graph_with_builtin_analysis_nodes<I, S>(names: I) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    let mut seen = HashSet::new();
    for name in names {
        let name = name.as_ref();
        let canonical_name = canonical_builtin_analysis_node_name(name).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                name.to_owned(),
            ))
        })?;
        if !seen.insert(canonical_name) {
            continue;
        }
        graph.push_boxed_node(boxed_analysis_node_by_name(canonical_name).ok_or_else(|| {
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
    let mut seen = HashSet::new();
    builtin_analysis_node_names()
        .iter()
        .filter_map(|name| canonical_builtin_analysis_node_name(name))
        .filter(|name| seen.insert(*name))
        .map(|name| {
            boxed_analysis_node_by_name(name)
                .unwrap_or_else(|| panic!("builtin analysis node should be registered: {name}"))
        })
        .collect()
}

pub fn graph_with_all_analysis_nodes() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
