use super::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState {
    pub events: ReplayStatsTimelineEvents,
}

pub struct StatsTimelineEventsNode {
    pub(super) state: StatsTimelineEventsState,
}

impl StatsTimelineEventsNode {
    pub fn new() -> Self {
        Self {
            state: StatsTimelineEventsState::default(),
        }
    }
}

impl Default for StatsTimelineEventsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for StatsTimelineEventsNode {
    type State = StatsTimelineEventsState;

    fn name(&self) -> &'static str {
        "stats_timeline_events"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        Self::dependencies()
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.capture_events(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}
