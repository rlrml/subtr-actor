use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct CeilingShotNode {
    calculator: CeilingShotCalculator,
}

impl CeilingShotNode {
    pub fn new() -> Self {
        Self {
            calculator: CeilingShotCalculator::new(),
        }
    }
}

impl Default for CeilingShotNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for CeilingShotNode {
    type State = CeilingShotCalculator;

    fn name(&self) -> &'static str {
        "ceiling_shot"
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
        self.calculator.update_parts(
            frame,
            ball,
            players,
            &touch_state.touch_events,
            live_play_state.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CeilingShotNode::new())
}
