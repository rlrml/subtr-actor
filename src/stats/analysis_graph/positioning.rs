use super::graph::*;
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
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            possession_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let possession_state = ctx.get::<PossessionState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
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
