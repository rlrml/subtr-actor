use super::graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BallFrameStateNode {
    state: BallFrameState,
}

impl BallFrameStateNode {
    pub fn new() -> Self {
        Self {
            state: BallFrameState::default(),
        }
    }
}

impl Default for BallFrameStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallFrameStateNode {
    type State = BallFrameState;

    fn name(&self) -> &'static str {
        "ball_frame_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.ball_frame_state();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallFrameStateNode::new())
}
