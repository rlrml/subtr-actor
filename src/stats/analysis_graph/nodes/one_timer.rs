use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct OneTimerNode {
    calculator: OneTimerCalculator,
}

impl OneTimerNode {
    pub fn new() -> Self {
        Self {
            calculator: OneTimerCalculator::new(),
        }
    }
}

impl Default for OneTimerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for OneTimerNode {
    type State = OneTimerCalculator;

    fn name(&self) -> &'static str {
        "one_timer"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            pass_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PassCalculator>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(OneTimerNode::new())
}
