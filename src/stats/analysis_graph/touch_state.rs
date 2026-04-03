use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct TouchStateNode {
    calculator: TouchStateCalculator,
    state: TouchState,
}

impl TouchStateNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchStateCalculator::new(),
            state: TouchState::default(),
        }
    }
}

impl Default for TouchStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for TouchStateNode {
    type State = TouchState;

    fn name(&self) -> &'static str {
        "touch_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
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
    Box::new(TouchStateNode::new())
}
