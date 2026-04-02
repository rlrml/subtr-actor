use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

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

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            frame_events_state_dependency(),
            backboard_bounce_state_dependency(),
            AnalysisDependency::required::<LivePlayState>(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<BackboardBounceState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DoubleTapNode::new())
}
