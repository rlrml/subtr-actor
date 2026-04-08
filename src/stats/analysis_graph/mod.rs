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

mod collector;
mod nodes;

use crate::stats::calculators::FrameInput;

#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use nodes::*;

pub(crate) fn boxed_analysis_node_by_name(name: &str) -> Option<Box<dyn AnalysisNodeDyn>> {
    match name {
        "core" => Some(nodes::match_stats::boxed_default()),
        "backboard" => Some(nodes::backboard::boxed_default()),
        "ceiling_shot" => Some(nodes::ceiling_shot::boxed_default()),
        "double_tap" => Some(nodes::double_tap::boxed_default()),
        "fifty_fifty" => Some(nodes::fifty_fifty::boxed_default()),
        "possession" => Some(nodes::possession::boxed_default()),
        "pressure" => Some(nodes::pressure::boxed_default()),
        "rush" => Some(nodes::rush::boxed_default()),
        "touch" => Some(nodes::touch::boxed_default()),
        "speed_flip" => Some(nodes::speed_flip::boxed_default()),
        "musty_flick" => Some(nodes::musty_flick::boxed_default()),
        "dodge_reset" => Some(nodes::dodge_reset::boxed_default()),
        "ball_carry" => Some(nodes::ball_carry::boxed_default()),
        "boost" => Some(nodes::boost::boxed_default()),
        "movement" => Some(nodes::movement::boxed_default()),
        "positioning" => Some(nodes::positioning::boxed_default()),
        "powerslide" => Some(nodes::powerslide::boxed_default()),
        "demo" => Some(nodes::demo::boxed_default()),
        _ => None,
    }
}

pub fn graph_with_builtin_analysis_nodes<I, S>(names: I) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(nodes::live_play::boxed_default());
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
        nodes::backboard::boxed_default(),
        nodes::ball_carry::boxed_default(),
        nodes::boost::boxed_default(),
        nodes::ceiling_shot::boxed_default(),
        nodes::demo::boxed_default(),
        nodes::dodge_reset::boxed_default(),
        nodes::double_tap::boxed_default(),
        nodes::fifty_fifty::boxed_default(),
        nodes::match_stats::boxed_default(),
        nodes::movement::boxed_default(),
        nodes::musty_flick::boxed_default(),
        nodes::positioning::boxed_default(),
        nodes::possession::boxed_default(),
        nodes::powerslide::boxed_default(),
        nodes::pressure::boxed_default(),
        nodes::rush::boxed_default(),
        nodes::settings::boxed_default(),
        nodes::speed_flip::boxed_default(),
        nodes::touch::boxed_default(),
    ]
}

pub fn graph_with_all_analysis_nodes() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(nodes::live_play::boxed_default());
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
