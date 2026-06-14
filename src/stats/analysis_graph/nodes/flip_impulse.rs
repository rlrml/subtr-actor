use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects dodges/flip impulses from player frame state (graph node name "dodge").
pub struct FlipImpulseNode {
    calculator: FlipImpulseCalculator,
}

impl FlipImpulseNode {
    pub fn new() -> Self {
        Self {
            calculator: FlipImpulseCalculator::new(),
        }
    }
}

impl Default for FlipImpulseNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FlipImpulseNode {
    type State = FlipImpulseCalculator;

    fn name(&self) -> &'static str {
        "dodge"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
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
    Box::new(FlipImpulseNode::new())
}
