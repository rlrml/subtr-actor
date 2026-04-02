use super::analysis_graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PlayerFrameStateNode {
    state: PlayerFrameState,
}

impl PlayerFrameStateNode {
    pub fn new() -> Self {
        Self {
            state: PlayerFrameState::default(),
        }
    }
}

impl Default for PlayerFrameStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PlayerFrameStateNode {
    type State = PlayerFrameState;

    fn name(&self) -> &'static str {
        "player_frame_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.player_frame_state();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PlayerFrameStateNode::new())
}
