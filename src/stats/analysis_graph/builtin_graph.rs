use std::collections::HashSet;

use crate::stats::calculators::FrameInput;
use crate::{Collector, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

use super::builtin_aliases::canonical_builtin_analysis_node_name;
use super::builtin_all_nodes::all_analysis_nodes;
use super::builtin_nodes::boxed_analysis_node_by_name;
use super::{collector, AnalysisGraph};

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

pub fn graph_with_all_analysis_nodes() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}
