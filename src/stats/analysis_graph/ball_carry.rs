use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BallCarryNode {
    calculator: BallCarryCalculator,
}

impl BallCarryNode {
    pub fn new() -> Self {
        Self {
            calculator: BallCarryCalculator::new(),
        }
    }
}

impl Default for BallCarryNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallCarryNode {
    type State = BallCarryCalculator;

    fn name(&self) -> &'static str {
        "ball_carry"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let touch_state = ctx.get::<TouchState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
            touch_state.last_touch_player.clone(),
        )
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.calculator.finish_calculation()
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallCarryNode::new())
}
