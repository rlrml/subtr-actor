use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects whiffs and beaten-to-ball attempts from ball/player state and touches during live play.
pub struct WhiffNode {
    calculator: WhiffCalculator,
}

impl WhiffNode {
    pub fn new() -> Self {
        Self {
            calculator: WhiffCalculator::new(),
        }
    }
}

impl Default for WhiffNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WhiffNode {
    type State = WhiffCalculator;

    fn name(&self) -> &'static str {
        "whiff"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::WHIFF_EMITTED_EVENTS
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

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WhiffNode::new())
}
