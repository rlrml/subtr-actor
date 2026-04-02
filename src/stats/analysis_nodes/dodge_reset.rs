use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct DodgeResetNode {
    calculator: DodgeResetCalculator,
}

impl DodgeResetNode {
    pub fn new() -> Self {
        Self {
            calculator: DodgeResetCalculator::new(),
        }
    }
}

impl Default for DodgeResetNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DodgeResetNode {
    type State = DodgeResetCalculator;

    fn name(&self) -> &'static str {
        "dodge_reset"
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
    Box::new(DodgeResetNode::new())
}
