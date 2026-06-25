use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Derives loose team possession (last team to touch owns the ball until the
/// opponent takes it away) directly from touch and live-play state.
pub struct LoosePossessionNode {
    calculator: LoosePossessionCalculator,
}

impl LoosePossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: LoosePossessionCalculator::new(),
        }
    }
}

impl Default for LoosePossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for LoosePossessionNode {
    type State = LoosePossessionCalculator;

    fn name(&self) -> &'static str {
        "loose_possession"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::LOOSE_POSSESSION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
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
    Box::new(LoosePossessionNode::new())
}
