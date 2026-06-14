use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects player-on-player bumps from player frame/events and 50/50 state.
pub struct BumpNode {
    calculator: BumpCalculator,
}

impl BumpNode {
    pub fn new() -> Self {
        Self {
            calculator: BumpCalculator::new(),
        }
    }
}

impl Default for BumpNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BumpNode {
    type State = BumpCalculator;

    fn name(&self) -> &'static str {
        "bump"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            fifty_fifty_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_with_fifty_fifty_state(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<FiftyFiftyState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BumpNode::new())
}
