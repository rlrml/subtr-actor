use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PositioningNode {
    calculator: PositioningCalculator,
}

impl PositioningNode {
    pub fn new() -> Self {
        Self::with_config(PositioningCalculatorConfig::default())
    }

    pub fn with_config(config: PositioningCalculatorConfig) -> Self {
        Self {
            calculator: PositioningCalculator::with_config(config),
        }
    }
}

impl Default for PositioningNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PositioningNode {
    type State = PositioningCalculator;

    fn name(&self) -> &'static str {
        "positioning"
    }

    fn dependencies(&self) -> NodeDependencies {
        let mut dependencies = vec![frame_state_dependency()];
        dependencies.push(possession_state_dependency());
        dependencies
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<FrameState>()?;
        let possession_state = ctx.get::<PossessionState>()?;
        self.calculator.update(
            sample,
            possession_state.active_player_before_sample.as_ref(),
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PositioningNode::new())
}
