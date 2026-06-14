use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Tracks which half of the field the ball is in from ball-frame and live-play state.
pub struct BallHalfNode {
    calculator: BallHalfCalculator,
}

impl BallHalfNode {
    pub fn new() -> Self {
        Self::with_config(BallHalfCalculatorConfig::default())
    }

    pub fn with_config(config: BallHalfCalculatorConfig) -> Self {
        Self {
            calculator: BallHalfCalculator::with_config(config),
        }
    }
}

impl Default for BallHalfNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallHalfNode {
    type State = BallHalfCalculator;

    fn name(&self) -> &'static str {
        "ball_half"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_event();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallHalfNode::new())
}
