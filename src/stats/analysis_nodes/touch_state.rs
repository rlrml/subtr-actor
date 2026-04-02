use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct TouchStateNode {
    calculator: TouchStateCalculator,
    state: TouchState,
}

impl TouchStateNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchStateCalculator::new(),
            state: TouchState::default(),
        }
    }
}

impl Default for TouchStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for TouchStateNode {
    type State = TouchState;

    fn name(&self) -> &'static str {
        "touch_state"
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
    Box::new(TouchStateNode::new())
}
