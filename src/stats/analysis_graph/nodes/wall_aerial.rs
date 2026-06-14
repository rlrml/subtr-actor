use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects wall aerials from ball/player positions and touches during live play.
pub struct WallAerialNode {
    calculator: WallAerialCalculator,
}

impl WallAerialNode {
    pub fn new() -> Self {
        Self {
            calculator: WallAerialCalculator::new(),
        }
    }
}

impl Default for WallAerialNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WallAerialNode {
    type State = WallAerialCalculator;

    fn name(&self) -> &'static str {
        "wall_aerial"
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
        let frame = ctx.get::<FrameInfo>()?;
        let ball = ctx.get::<BallFrameState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator
            .update(frame, ball, players, touch_state, live_play_state)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WallAerialNode::new())
}
