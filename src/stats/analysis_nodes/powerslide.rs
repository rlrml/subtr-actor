use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PowerslideNode {
    calculator: PowerslideCalculator,
}

impl PowerslideNode {
    pub fn new() -> Self {
        Self {
            calculator: PowerslideCalculator::new(),
        }
    }
}

impl Default for PowerslideNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PowerslideNode {
    type State = PowerslideCalculator;

    fn name(&self) -> &'static str {
        "powerslide"
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
    Box::new(PowerslideNode::new())
}
