use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FrameStateNode {
    state: FrameState,
}

impl FrameStateNode {
    pub fn new() -> Self {
        Self {
            state: FrameState::default(),
        }
    }
}

impl Default for FrameStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FrameStateNode {
    type State = FrameState;

    fn name(&self) -> &'static str {
        "frame_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = FrameState::from_parts(
            ctx.get::<FrameInfo>()?.clone(),
            ctx.get::<GameplayState>()?.clone(),
            ctx.get::<BallFrameState>()?.clone(),
            ctx.get::<PlayerFrameState>()?.clone(),
            ctx.get::<FrameEventsState>()?.clone(),
        );
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FrameStateNode::new())
}
