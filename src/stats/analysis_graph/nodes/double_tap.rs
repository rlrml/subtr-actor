use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects double taps from touches plus backboard-bounce state during live play.
pub struct DoubleTapNode {
    calculator: DoubleTapCalculator,
}

impl DoubleTapNode {
    pub fn new() -> Self {
        Self {
            calculator: DoubleTapCalculator::new(),
        }
    }
}

impl Default for DoubleTapNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DoubleTapNode {
    type State = DoubleTapCalculator;

    fn name(&self) -> &'static str {
        "double_tap"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::DOUBLE_TAP_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            backboard_bounce_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<BackboardBounceState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DoubleTapNode::new())
}
