use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct DemoNode {
    calculator: DemoCalculator,
}

impl DemoNode {
    pub fn new() -> Self {
        Self {
            calculator: DemoCalculator::new(),
        }
    }
}

impl Default for DemoNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DemoNode {
    type State = DemoCalculator;

    fn name(&self) -> &'static str {
        "demo"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DemoNode::new())
}
