use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct SpeedFlipNode {
    calculator: SpeedFlipCalculator,
}

impl SpeedFlipNode {
    pub fn new() -> Self {
        Self {
            calculator: SpeedFlipCalculator::new(),
        }
    }
}

impl Default for SpeedFlipNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for SpeedFlipNode {
    type State = SpeedFlipCalculator;

    fn name(&self) -> &'static str {
        "speed_flip"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(SpeedFlipNode::new())
}
