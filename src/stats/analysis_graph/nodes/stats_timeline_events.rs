use super::*;
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
        aerial_goal_dependency(),
        high_aerial_goal_dependency(),
        long_distance_goal_dependency(),
        own_half_goal_dependency(),
        empty_net_goal_dependency(),
        rush_dependency(),
        speed_flip_dependency(),
        wavedash_dependency(),
        whiff_dependency(),
        boost_dependency(),
    ],
    inputs = {
        match_stats: MatchStatsCalculator,
        demo: DemoCalculator,
        backboard: BackboardCalculator,
        ceiling_shot: CeilingShotCalculator,
        double_tap: DoubleTapCalculator,
        fifty_fifty: FiftyFiftyCalculator,
        aerial_goal: AerialGoalCalculator,
        high_aerial_goal: HighAerialGoalCalculator,
        long_distance_goal: LongDistanceGoalCalculator,
        own_half_goal: OwnHalfGoalCalculator,
        empty_net_goal: EmptyNetGoalCalculator,
        rush: RushCalculator,
        speed_flip: SpeedFlipCalculator,
        wavedash: WavedashCalculator,
        whiff: WhiffCalculator,
        boost: BoostCalculator,
    },
    evaluate = |node| {
        let mut timeline = match_stats.timeline().to_vec();
        timeline.extend(demo.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));
        let goal_tags = combined_goal_tag_events(&[
            aerial_goal.events(),
            high_aerial_goal.events(),
            long_distance_goal.events(),
            own_half_goal.events(),
            empty_net_goal.events(),
        ]);

        node.state.events = ReplayStatsTimelineEvents {
            timeline,
            goal_context: match_stats.goal_context_events().to_vec(),
            backboard: backboard.events().to_vec(),
            ceiling_shot: ceiling_shot.events().to_vec(),
            double_tap: double_tap.events().to_vec(),
            fifty_fifty: fifty_fifty.events().to_vec(),
            goal_tags,
            rush: rush.events().to_vec(),
            speed_flip: speed_flip.events().to_vec(),
            wavedash: wavedash.events().to_vec(),
            whiff: whiff.events().to_vec(),
            boost_pickups: boost.pickup_comparison_events().to_vec(),
        };
        Ok(())
    },
    state_ref = |node| &node.state,
}
