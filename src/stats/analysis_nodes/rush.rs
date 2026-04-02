use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct RushNode {
    calculator: RushCalculator,
}

impl RushNode {
    pub fn new() -> Self {
        Self::with_config(RushCalculatorConfig::default())
    }

    pub fn with_config(config: RushCalculatorConfig) -> Self {
        Self {
            calculator: RushCalculator::with_config(config),
        }
    }
}

impl Default for RushNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for RushNode {
    type State = RushCalculator;

    fn name(&self) -> &'static str {
        "rush"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            possession_state_dependency(),
            AnalysisDependency::required::<LivePlayState>(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let possession_state = ctx.get::<PossessionState>()?;
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
            possession_state,
        )
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.calculator.finish_calculation()
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(RushNode::new())
}
