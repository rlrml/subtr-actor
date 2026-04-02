use super::analysis_graph::*;
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

impl Default for FrameEventsStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FrameEventsStateNode {
    type State = FrameEventsState;

    fn name(&self) -> &'static str {
        "frame_events_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.frame_events_state();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FrameEventsStateNode::new())
}
