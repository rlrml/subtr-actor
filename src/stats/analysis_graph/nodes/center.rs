use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct CenterNode {
    calculator: CenterCalculator,
}

impl CenterNode {
    pub fn new() -> Self {
        Self {
            calculator: CenterCalculator::new(),
        }
    }
}

impl Default for CenterNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for CenterNode {
    type State = CenterCalculator;

    fn name(&self) -> &'static str {
        "center"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::CENTER_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            touch_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CenterNode::new())
}
