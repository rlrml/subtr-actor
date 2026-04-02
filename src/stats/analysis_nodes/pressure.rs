use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PressureNode {
    calculator: PressureCalculator,
}

impl PressureNode {
    pub fn new() -> Self {
        Self::with_config(PressureCalculatorConfig::default())
    }

    pub fn with_config(config: PressureCalculatorConfig) -> Self {
        Self {
            calculator: PressureCalculator::with_config(config),
        }
    }
}

impl Default for PressureNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PressureNode {
    type State = PressureCalculator;

    fn name(&self) -> &'static str {
        "pressure"
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
    Box::new(PressureNode::new())
}
