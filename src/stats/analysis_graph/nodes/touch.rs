use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Classifies ball touches (with rotation/possession/50-50/vertical context) into touch events/stats.
pub struct TouchNode {
    calculator: TouchCalculator,
}

impl TouchNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchCalculator::new(),
        }
    }
}

impl Default for TouchNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for TouchNode {
    type State = TouchCalculator;

    fn name(&self) -> &'static str {
        "touch"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            player_vertical_state_dependency(),
            rotation_dependency(),
            touch_state_dependency(),
            possession_state_dependency(),
            fifty_fifty_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let touch_state = ctx.get::<TouchState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerVerticalState>()?,
            ctx.get::<RotationCalculator>()?,
            touch_state,
            ctx.get::<PossessionState>()?,
            ctx.get::<FiftyFiftyState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(TouchNode::new())
}
