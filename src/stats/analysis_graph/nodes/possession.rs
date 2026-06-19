use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Derives team/possession stats from ball-frame and shared possession state.
pub struct PossessionNode {
    calculator: PossessionCalculator,
}

impl PossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionCalculator::new(),
        }
    }
}

impl Default for PossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PossessionNode {
    type State = PossessionCalculator;

    fn name(&self) -> &'static str {
        "possession"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::POSSESSION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            possession_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PossessionState>()?,
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
    Box::new(PossessionNode::new())
}
