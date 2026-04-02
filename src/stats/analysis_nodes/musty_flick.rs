use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MustyFlickNode {
    calculator: MustyFlickCalculator,
}

impl MustyFlickNode {
    pub fn new() -> Self {
        Self {
            calculator: MustyFlickCalculator::new(),
        }
    }
}

impl Default for MustyFlickNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MustyFlickNode {
    type State = MustyFlickCalculator;

    fn name(&self) -> &'static str {
        "musty_flick"
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
    Box::new(MustyFlickNode::new())
}
