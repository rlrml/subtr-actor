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
        vec![core_sample_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        self.calculator.update(sample)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MovementNode::new())
}
