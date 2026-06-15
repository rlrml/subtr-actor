use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FiftyFiftyNode {
    calculator: FiftyFiftyCalculator,
}

impl FiftyFiftyNode {
    pub fn new() -> Self {
        Self {
            calculator: FiftyFiftyCalculator::new(),
        }
    }
}

impl Default for FiftyFiftyNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FiftyFiftyNode {
    type State = FiftyFiftyCalculator;

    fn name(&self) -> &'static str {
        "fifty_fifty"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::FIFTY_FIFTY_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![fifty_fifty_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let fifty_fifty_state = ctx.get::<FiftyFiftyState>()?;
        self.calculator.update(fifty_fifty_state)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FiftyFiftyNode::new())
}
