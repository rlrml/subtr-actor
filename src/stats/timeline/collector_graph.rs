use crate::stats::analysis_graph::{
    AnalysisGraph, StatsTimelineEventsNode, StatsTimelineFrameNode,
};
use crate::*;

pub fn build_legacy_timeline_graph() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(StatsTimelineFrameNode::new()));
    graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
    graph
}

#[deprecated(
    note = "use build_legacy_timeline_graph for full partial-sum snapshots, or build_timeline_event_graph for compact event-backed timelines"
)]
pub fn build_timeline_graph() -> AnalysisGraph {
    build_legacy_timeline_graph()
}

pub fn build_timeline_event_graph() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
    graph
}
