use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
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

    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(projected_timeline_events(&self.calculator))
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the streams'
/// interim lifecycle rules.
fn projected_timeline_events(calculator: &MatchStatsCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Timeline scoreboard/goal moments are committed complete and never
    // mutated afterwards (goals join the list only once attributed). The
    // calculator re-sorts its list by time when a pending goal flushes, but
    // same-anchor entries share a time, so the stable sort preserves their
    // commit order (see the id-determinism notes on `EventAssembler`).
    for event in calculator.timeline() {
        let frame = event.frame.unwrap_or_default();
        assembler.push(
            "timeline",
            frame,
            EventLifecycle::Finalized,
            moment(frame, event.time),
            EventPayload::Timeline(event.clone()),
            event.player_id.clone(),
            None,
            event.is_team_0,
            event.player_position,
            None,
            None,
        );
    }

    // Scoreboard-delta moments: committed complete, never revised.
    for event in calculator.core_player_events() {
        assembler.push(
            "core_player",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::CorePlayer(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MatchStatsNode::new())
}
