use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct WallAerialShotNode {
    calculator: WallAerialShotCalculator,
}

impl WallAerialShotNode {
    pub fn new() -> Self {
        Self {
            calculator: WallAerialShotCalculator::new(),
        }
    }
}

impl Default for WallAerialShotNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WallAerialShotNode {
    type State = WallAerialShotCalculator;

    fn name(&self) -> &'static str {
        "wall_aerial_shot"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let frame_events = ctx.get::<FrameEventsState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator
            .update(frame, players, frame_events, live_play_state)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WallAerialShotNode::new())
}
