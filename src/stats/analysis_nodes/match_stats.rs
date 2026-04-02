use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MatchStatsNode {
    calculator: MatchStatsCalculator,
}

impl MatchStatsNode {
    pub fn new() -> Self {
        Self {
            calculator: MatchStatsCalculator::new(),
        }
    }
}

impl Default for MatchStatsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MatchStatsNode {
    type State = MatchStatsCalculator;

    fn name(&self) -> &'static str {
        "match_stats"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?.is_live_play,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MatchStatsNode::new())
}
