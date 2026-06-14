use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects half-volleys from ball/player state and touches during live play.
pub struct HalfVolleyNode {
    calculator: HalfVolleyCalculator,
}

impl HalfVolleyNode {
    pub fn new() -> Self {
        Self {
            calculator: HalfVolleyCalculator::new(),
        }
    }
}

impl Default for HalfVolleyNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for HalfVolleyNode {
    type State = HalfVolleyCalculator;

    fn name(&self) -> &'static str {
        "half_volley"
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
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HalfVolleyNode::new())
}
