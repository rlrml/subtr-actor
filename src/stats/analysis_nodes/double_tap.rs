use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct DoubleTapNode {
    calculator: DoubleTapCalculator,
}

impl DoubleTapNode {
    pub fn new() -> Self {
        Self {
            calculator: DoubleTapCalculator::new(),
        }
    }
}

impl Default for DoubleTapNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DoubleTapNode {
    type State = DoubleTapCalculator;

    fn name(&self) -> &'static str {
        "double_tap"
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
    Box::new(DoubleTapNode::new())
}
