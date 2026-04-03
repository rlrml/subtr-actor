use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState {
    pub events: ReplayStatsTimelineEvents,
}

pub struct StatsTimelineEventsNode {
    state: StatsTimelineEventsState,
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

    fn dependencies(&self) -> NodeDependencies {
        vec![
            match_stats_dependency(),
            demo_dependency(),
            backboard_dependency(),
            ceiling_shot_dependency(),
            double_tap_dependency(),
            fifty_fifty_dependency(),
            rush_dependency(),
            speed_flip_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let mut timeline = ctx.get::<MatchStatsCalculator>()?.timeline().to_vec();
        timeline.extend(ctx.get::<DemoCalculator>()?.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));

        self.state.events = ReplayStatsTimelineEvents {
            timeline,
            backboard: ctx.get::<BackboardCalculator>()?.events().to_vec(),
            ceiling_shot: ctx.get::<CeilingShotCalculator>()?.events().to_vec(),
            double_tap: ctx.get::<DoubleTapCalculator>()?.events().to_vec(),
            fifty_fifty: ctx.get::<FiftyFiftyCalculator>()?.events().to_vec(),
            rush: ctx.get::<RushCalculator>()?.events().to_vec(),
            speed_flip: ctx.get::<SpeedFlipCalculator>()?.events().to_vec(),
        };
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineEventsNode::new())
}
