use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MovementNode {
    calculator: MovementCalculator,
}

impl MovementNode {
    pub fn new() -> Self {
        Self {
            calculator: MovementCalculator::new(),
        }
    }
}

impl Default for MovementNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MovementNode {
    type State = MovementCalculator;

    fn name(&self) -> &'static str {
        "movement"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            player_vertical_state_dependency(),
            AnalysisDependency::required::<LivePlayState>(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerVerticalState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MovementNode::new())
}
