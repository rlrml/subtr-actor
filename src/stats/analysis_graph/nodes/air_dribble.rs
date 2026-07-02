use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects air dribbles from continuous same-player non-ground touches.
pub struct AirDribbleNode {
    calculator: AirDribbleCalculator,
}

impl AirDribbleNode {
    pub fn new() -> Self {
        Self {
            calculator: AirDribbleCalculator::new(),
        }
    }
}

impl Default for AirDribbleNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for AirDribbleNode {
    type State = AirDribbleCalculator;

    fn name(&self) -> &'static str {
        "air_dribble"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::AIR_DRIBBLE_EMITTED_EVENTS
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
            touch_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?,
            ctx.get::<TouchCalculator>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish()
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(AirDribbleNode::new())
}
