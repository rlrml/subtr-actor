use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct ControlledPlayNode {
    calculator: ControlledPlayCalculator,
}

impl ControlledPlayNode {
    pub fn new() -> Self {
        Self {
            calculator: ControlledPlayCalculator::new(),
        }
    }
}

impl Default for ControlledPlayNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for ControlledPlayNode {
    type State = ControlledPlayCalculator;

    fn name(&self) -> &'static str {
        "controlled_play"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<TouchState>()?,
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
    Box::new(ControlledPlayNode::new())
}
