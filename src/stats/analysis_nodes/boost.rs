use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BoostNode {
    calculator: BoostCalculator,
}

impl BoostNode {
    pub fn new() -> Self {
        Self::with_config(BoostCalculatorConfig::default())
    }

    pub fn with_config(config: BoostCalculatorConfig) -> Self {
        Self {
            calculator: BoostCalculator::with_config(config),
        }
    }
}

impl Default for BoostNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BoostNode {
    type State = BoostCalculator;

    fn name(&self) -> &'static str {
        "boost"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![frame_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<FrameState>()?;
        self.calculator.update(sample)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BoostNode::new())
}
