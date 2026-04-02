use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct LivePlayNode {
    tracker: LivePlayTracker,
    state: LivePlayState,
}

impl LivePlayNode {
    pub fn new() -> Self {
        Self {
            tracker: LivePlayTracker::default(),
            state: LivePlayState::default(),
        }
    }
}

impl Default for LivePlayNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for LivePlayNode {
    type State = LivePlayState;

    fn name(&self) -> &'static str {
        "live_play"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![gameplay_state_dependency(), frame_events_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state.is_live_play = self
            .tracker
            .is_live_play_parts(ctx.get::<GameplayState>()?, ctx.get::<FrameEventsState>()?);
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(LivePlayNode::new())
}
