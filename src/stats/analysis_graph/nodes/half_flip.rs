use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects half-flips from player frame state during live play.
pub struct HalfFlipNode {
    calculator: HalfFlipCalculator,
}

impl HalfFlipNode {
    pub fn new() -> Self {
        Self {
            calculator: HalfFlipCalculator::new(),
        }
    }
}

impl Default for HalfFlipNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for HalfFlipNode {
    type State = HalfFlipCalculator;

    fn name(&self) -> &'static str {
        "half_flip"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finalize(ctx.get::<FrameInfo>()?);
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HalfFlipNode::new())
}
