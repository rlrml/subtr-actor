use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BackboardBounceStateNode {
    calculator: BackboardBounceCalculator,
    state: BackboardBounceState,
}

impl BackboardBounceStateNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardBounceCalculator::new(),
            state: BackboardBounceState::default(),
        }
    }
}

impl Default for BackboardBounceStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BackboardBounceStateNode {
    type State = BackboardBounceState;

    fn name(&self) -> &'static str {
        "backboard_bounce_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        );
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BackboardBounceStateNode::new())
}
