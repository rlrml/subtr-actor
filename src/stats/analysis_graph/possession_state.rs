use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PossessionStateNode {
    calculator: PossessionStateCalculator,
    state: PossessionState,
}

impl PossessionStateNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionStateCalculator::new(),
            state: PossessionState::default(),
        }
    }
}

impl Default for PossessionStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PossessionStateNode {
    type State = PossessionState;

    fn name(&self) -> &'static str {
        "possession_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let touch_state = ctx.get::<TouchState>()?;
        self.state = self.calculator.update(
            ctx.get::<FrameInfo>()?,
            touch_state,
            ctx.get::<LivePlayState>()?.is_live_play,
        );
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PossessionStateNode::new())
}
