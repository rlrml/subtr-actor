use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PlayerVerticalStateNode {
    calculator: PlayerVerticalStateCalculator,
    state: PlayerVerticalState,
}

impl PlayerVerticalStateNode {
    pub fn new() -> Self {
        Self {
            calculator: PlayerVerticalStateCalculator::new(),
            state: PlayerVerticalState::default(),
        }
    }
}

impl Default for PlayerVerticalStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PlayerVerticalStateNode {
    type State = PlayerVerticalState;

    fn name(&self) -> &'static str {
        "player_vertical_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![player_frame_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = self.calculator.update(ctx.get::<PlayerFrameState>()?);
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PlayerVerticalStateNode::new())
}
