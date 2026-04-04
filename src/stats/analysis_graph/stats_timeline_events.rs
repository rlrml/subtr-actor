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

impl_analysis_node! {
    node = StatsTimelineEventsNode,
    state = StatsTimelineEventsState,
    name = "stats_timeline_events",
    dependencies = [
        match_stats_dependency(),
        demo_dependency(),
        backboard_dependency(),
        ceiling_shot_dependency(),
        double_tap_dependency(),
        fifty_fifty_dependency(),
        rush_dependency(),
        speed_flip_dependency(),
    ],
    inputs = {
        match_stats: MatchStatsCalculator,
        demo: DemoCalculator,
        backboard: BackboardCalculator,
        ceiling_shot: CeilingShotCalculator,
        double_tap: DoubleTapCalculator,
        fifty_fifty: FiftyFiftyCalculator,
        rush: RushCalculator,
        speed_flip: SpeedFlipCalculator,
    },
    evaluate = |node| {
        let mut timeline = match_stats.timeline().to_vec();
        timeline.extend(demo.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));

        node.state.events = ReplayStatsTimelineEvents {
            timeline,
            backboard: backboard.events().to_vec(),
            ceiling_shot: ceiling_shot.events().to_vec(),
            double_tap: double_tap.events().to_vec(),
            fifty_fifty: fifty_fifty.events().to_vec(),
            rush: rush.events().to_vec(),
            speed_flip: speed_flip.events().to_vec(),
        };
        Ok(())
    },
    state_ref = |node| &node.state,
}
