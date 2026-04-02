use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct TouchNode {
    calculator: TouchCalculator,
}

impl TouchNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchCalculator::new(),
        }
    }
}

impl Default for TouchNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for TouchNode {
    type State = TouchCalculator;

    fn name(&self) -> &'static str {
        "touch"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![core_sample_dependency(), touch_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        let touch_state = ctx.get::<TouchState>()?;
        self.calculator.update(sample, touch_state)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(TouchNode::new())
}
