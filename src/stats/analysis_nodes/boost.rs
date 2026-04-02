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
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            player_vertical_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<PlayerVerticalState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BoostNode::new())
}
