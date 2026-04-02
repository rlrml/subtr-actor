use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FiftyFiftyStateNode {
    calculator: FiftyFiftyStateCalculator,
    state: FiftyFiftyState,
}

impl FiftyFiftyStateNode {
    pub fn new() -> Self {
        Self {
            calculator: FiftyFiftyStateCalculator::new(),
            state: FiftyFiftyState::default(),
        }
    }
}

impl Default for FiftyFiftyStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FiftyFiftyStateNode {
    type State = FiftyFiftyState;

    fn name(&self) -> &'static str {
        "fifty_fifty_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            core_sample_dependency(),
            touch_state_dependency(),
            possession_state_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let possession_state = ctx.get::<PossessionState>()?;
        self.state = self
            .calculator
            .update(sample, touch_state, possession_state);
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FiftyFiftyStateNode::new())
}
