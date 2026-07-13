use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PlayerControlStateNode {
    state: PlayerControlState,
}

impl PlayerControlStateNode {
    pub fn new() -> Self {
        Self {
            state: PlayerControlState::default(),
        }
    }
}

impl Default for PlayerControlStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PlayerControlStateNode {
    type State = PlayerControlState;

    fn name(&self) -> &'static str {
        "player_control_state"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.player_control_state();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(super) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PlayerControlStateNode::new())
}
