use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct CeilingShotNode {
    calculator: CeilingShotCalculator,
}

impl CeilingShotNode {
    pub fn new() -> Self {
        Self {
            calculator: CeilingShotCalculator::new(),
        }
    }
}

impl Default for CeilingShotNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for CeilingShotNode {
    type State = CeilingShotCalculator;

    fn name(&self) -> &'static str {
        "ceiling_shot"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![core_sample_dependency(), touch_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        let touch_state = ctx.get::<TouchState>()?;
        self.calculator.update(sample, &touch_state.touch_events)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CeilingShotNode::new())
}
