use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PossessionNode {
    calculator: PossessionCalculator,
}

impl PossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionCalculator::new(),
        }
    }
}

impl Default for PossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PossessionNode {
    type State = PossessionCalculator;

    fn name(&self) -> &'static str {
        "possession"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![core_sample_dependency(), possession_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        let possession_state = ctx.get::<PossessionState>()?;
        self.calculator
            .update(sample, possession_state.active_team_before_sample)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PossessionNode::new())
}
