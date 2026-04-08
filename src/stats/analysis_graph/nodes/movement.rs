use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MovementNode {
    calculator: MovementCalculator,
}

impl MovementNode {
    pub fn new() -> Self {
        Self {
            calculator: MovementCalculator::new(),
        }
    }
}

impl Default for MovementNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MovementNode {
    type State = MovementCalculator;

    fn name(&self) -> &'static str {
        "movement"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            player_vertical_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerVerticalState>()?,
            live_play_state.counts_toward_player_motion(),
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MovementNode::new())
}
