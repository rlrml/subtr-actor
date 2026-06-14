use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects wavedashes from player frame state during live play.
pub struct WavedashNode {
    calculator: WavedashCalculator,
}

impl WavedashNode {
    pub fn new() -> Self {
        Self {
            calculator: WavedashCalculator::new(),
        }
    }
}

impl Default for WavedashNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WavedashNode {
    type State = WavedashCalculator;

    fn name(&self) -> &'static str {
        "wavedash"
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

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WavedashNode::new())
}
