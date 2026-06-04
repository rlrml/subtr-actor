use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct RotationNode {
    calculator: RotationCalculator,
}

impl RotationNode {
    pub fn new() -> Self {
        Self::with_config(RotationCalculatorConfig::default())
    }

    pub fn with_config(config: RotationCalculatorConfig) -> Self {
        Self {
            calculator: RotationCalculator::with_config(config),
        }
    }
}

impl Default for RotationNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for RotationNode {
    type State = RotationCalculator;

    fn name(&self) -> &'static str {
        "rotation"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_player_events();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(RotationNode::new())
}
