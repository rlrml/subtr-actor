use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PassNode {
    calculator: PassCalculator,
}

impl PassNode {
    pub fn new() -> Self {
        Self {
            calculator: PassCalculator::new(),
        }
    }
}

impl Default for PassNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PassNode {
    type State = PassCalculator;

    fn name(&self) -> &'static str {
        "pass"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::PASS_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            touch_state_dependency(),
            backboard_bounce_state_dependency(),
            fifty_fifty_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<BackboardBounceState>()?,
            ctx.get::<FiftyFiftyState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PassNode::new())
}
