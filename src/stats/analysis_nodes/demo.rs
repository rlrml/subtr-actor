use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct DemoNode {
    calculator: DemoCalculator,
}

impl DemoNode {
    pub fn new() -> Self {
        Self {
            calculator: DemoCalculator::new(),
        }
    }
}

impl Default for DemoNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DemoNode {
    type State = DemoCalculator;

    fn name(&self) -> &'static str {
        "demo"
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
    Box::new(DemoNode::new())
}
