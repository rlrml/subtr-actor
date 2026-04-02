use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BackboardNode {
    calculator: BackboardCalculator,
}

impl BackboardNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardCalculator::new(),
        }
    }
}

impl Default for BackboardNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BackboardNode {
    type State = BackboardCalculator;

    fn name(&self) -> &'static str {
        "backboard"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            core_sample_dependency(),
            backboard_bounce_state_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        let backboard_bounce_state = ctx.get::<BackboardBounceState>()?;
        self.calculator.update(sample, backboard_bounce_state)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BackboardNode::new())
}
