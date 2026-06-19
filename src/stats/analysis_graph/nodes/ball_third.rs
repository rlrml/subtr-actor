use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BallThirdNode {
    calculator: BallThirdCalculator,
}

impl BallThirdNode {
    pub fn new() -> Self {
        Self::with_config(BallThirdCalculatorConfig::default())
    }

    pub fn with_config(config: BallThirdCalculatorConfig) -> Self {
        Self {
            calculator: BallThirdCalculator::with_config(config),
        }
    }
}

impl Default for BallThirdNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallThirdNode {
    type State = BallThirdCalculator;

    fn name(&self) -> &'static str {
        "ball_third"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BALL_THIRD_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_event();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallThirdNode::new())
}
