use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FiftyFiftyStateNode {
    calculator: FiftyFiftyStateCalculator,
    state: FiftyFiftyState,
}

impl FiftyFiftyStateNode {
    pub fn new() -> Self {
        Self {
            calculator: FiftyFiftyStateCalculator::new(),
            state: FiftyFiftyState::default(),
        }
    }
}

impl Default for FiftyFiftyStateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FiftyFiftyStateNode {
    type State = FiftyFiftyState;

    fn name(&self) -> &'static str {
        "fifty_fifty_state"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            possession_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let gameplay = ctx.get::<GameplayState>()?;
        let ball = ctx.get::<BallFrameState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let possession_state = ctx.get::<PossessionState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.state = self.calculator.update(
            frame,
            gameplay,
            ball,
            players,
            touch_state,
            possession_state,
            live_play_state.is_live_play,
        );
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FiftyFiftyStateNode::new())
}
