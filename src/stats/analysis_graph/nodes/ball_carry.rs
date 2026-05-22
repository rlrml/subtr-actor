use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BallCarryNode {
    calculator: BallCarryCalculator,
}

impl BallCarryNode {
    pub fn new() -> Self {
        Self {
            calculator: BallCarryCalculator::new(),
        }
    }

    fn update_from_control_state(
        &mut self,
        ctx: &AnalysisStateContext<'_>,
    ) -> SubtrActorResult<()> {
        self.calculator
            .update(ctx.get::<ContinuousBallControlState>()?)
    }
}

impl Default for BallCarryNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallCarryNode {
    type State = BallCarryCalculator;

    fn name(&self) -> &'static str {
        "ball_carry"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![continuous_ball_control_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_from_control_state(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_from_control_state(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallCarryNode::new())
}
