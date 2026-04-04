use super::graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FrameEventsStateNode {
    state: FrameEventsState,
}

impl FrameEventsStateNode {
    pub fn new() -> Self {
        Self {
            state: FrameEventsState::default(),
        }
    }
}

impl_analysis_node! {
    node = FrameEventsStateNode,
    state = FrameEventsState,
    name = "frame_events_state",
    dependencies = [AnalysisDependency::required::<FrameInput>()],
    inputs = { frame_input: FrameInput },
    evaluate = |node| {
        node.state = frame_input.frame_events_state();
        Ok(())
    },
    state_ref = |node| &node.state,
}
