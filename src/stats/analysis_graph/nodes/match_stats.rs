use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Accumulates match-level stats and goal contexts; attaches per-goal territorial pressure at finish.
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

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::MATCH_STATS_EMITTED_EVENTS
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
            touch_state_dependency(),
            // Not consumed per-frame; needed at finish to attach per-goal pressure.
            territorial_pressure_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?,
            ctx.get::<TouchState>()?,
        )
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        // Finalize goal contexts first, then attach pressure duration from the
        // now-final territorial-pressure sessions.
        self.calculator.finish()?;
        let territorial_pressure = ctx.get::<TerritorialPressureCalculator>()?;
        self.calculator
            .attach_goal_pressure_durations(&territorial_pressure.projected_events());
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MatchStatsNode::new())
}
