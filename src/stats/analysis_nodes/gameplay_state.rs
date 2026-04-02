use super::analysis_graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct GameplayStateNode {
    state: GameplayState,
}

impl GameplayStateNode {
    pub fn new() -> Self {
        Self {
            state: GameplayState::default(),
        }
    }
}

impl Default for GameplayStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for GameplayStateNode {
    type State = GameplayState;

    fn name(&self) -> &'static str {
        "gameplay_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.gameplay_state();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(GameplayStateNode::new())
}
