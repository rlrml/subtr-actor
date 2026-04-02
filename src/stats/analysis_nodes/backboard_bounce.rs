use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BackboardBounceStateNode {
    calculator: BackboardBounceCalculator,
    state: BackboardBounceState,
}

impl BackboardBounceStateNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardBounceCalculator::new(),
            state: BackboardBounceState::default(),
        }
    }
}

impl Default for BackboardBounceStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BackboardBounceStateNode {
    type State = BackboardBounceState;

    fn name(&self) -> &'static str {
        "backboard_bounce_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![core_sample_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        self.state = self.calculator.update(sample);
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BackboardBounceStateNode::new())
}
