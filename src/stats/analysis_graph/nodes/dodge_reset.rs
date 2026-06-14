use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects flip/dodge resets and their outcomes from player, ball, touch, and event state.
pub struct DodgeResetNode {
    calculator: DodgeResetCalculator,
}

impl DodgeResetNode {
    pub fn new() -> Self {
        Self {
            calculator: DodgeResetCalculator::new(),
        }
    }
}

impl Default for DodgeResetNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DodgeResetNode {
    type State = DodgeResetCalculator;

    fn name(&self) -> &'static str {
        "dodge_reset"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DodgeResetNode::new())
}
