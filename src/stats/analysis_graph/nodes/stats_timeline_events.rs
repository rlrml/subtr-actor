use super::*;
use crate::*;

/// Marker state for the timeline-events aggregation root.
///
/// The events themselves live in the graph-owned transaction log: interim
/// consumers read [`AnalysisGraph::event_transaction_log`] (fed by
/// [`AnalysisGraph::project_events_now`]), and batch consumers read the same
/// log's reduced view after [`AnalysisGraph::finish`]'s single
/// finalize-everything projection.
#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState;

const MECHANIC_AIR_DRIBBLE: &str = "air_dribble";
const MECHANIC_BALL_CARRY: &str = "ball_carry";
const MECHANIC_CEILING_SHOT: &str = "ceiling_shot";
const MECHANIC_CENTER: &str = "center";
const MECHANIC_DOUBLE_TAP: &str = "double_tap";
const MECHANIC_FLICK: &str = "flick";
const MECHANIC_FLIP_RESET: &str = "flip_reset";
const MECHANIC_HALF_FLIP: &str = "half_flip";
const MECHANIC_HALF_VOLLEY: &str = "half_volley";
const MECHANIC_ONE_TIMER: &str = "one_timer";
const MECHANIC_PASS: &str = "pass";
const MECHANIC_SPEED_FLIP: &str = "speed_flip";
const MECHANIC_WALL_AERIAL: &str = "wall_aerial";
const MECHANIC_WALL_AERIAL_SHOT: &str = "wall_aerial_shot";
const MECHANIC_WAVEDASH: &str = "wavedash";

/// List of mechanic kind identifiers emitted into the stats timeline.
pub const STATS_TIMELINE_MECHANIC_KINDS: &[&str] = &[
    MECHANIC_AIR_DRIBBLE,
    MECHANIC_BALL_CARRY,
    MECHANIC_CEILING_SHOT,
    MECHANIC_CENTER,
    MECHANIC_DOUBLE_TAP,
    MECHANIC_FLICK,
    MECHANIC_FLIP_RESET,
    MECHANIC_HALF_FLIP,
    MECHANIC_HALF_VOLLEY,
    MECHANIC_ONE_TIMER,
    MECHANIC_PASS,
    MECHANIC_SPEED_FLIP,
    MECHANIC_WALL_AERIAL,
    MECHANIC_WALL_AERIAL_SHOT,
    MECHANIC_WAVEDASH,
];

/// Aggregation root for the stats-timeline event surface.
///
/// Every event stream is projected by the analysis node that owns its
/// calculator (see `AnalysisNode::project_events` and the per-node
/// `projected_timeline_events` functions); this node projects nothing itself.
/// It exists so that pushing one node assembles the entire event graph — its
/// dependency list is the full set of event-projecting node states — and so
/// the `stats_timeline_events` name remains addressable for graph
/// introspection (ASCII DAG, analysis-node JSON dumps).
///
/// Event identity is cadence-invariant (see `EventAssembler`), so a
/// projection cadence only decides *when* an event becomes observable, never
/// *which* id it gets: a projection emits every event the calculators have
/// committed so far, marking each one
/// [`Confirmed`](EventLifecycle::Confirmed) while its content may still be
/// revised (an open span's growing end, a touch awaiting outcome enrichment,
/// ...) and [`Finalized`](EventLifecycle::Finalized) once no future evidence
/// can change it. The graph's transaction log turns successive projections
/// into `Upsert` transactions and enforces the lifecycle invariants
/// (finalized events never change or vanish, confirmed events never vanish).
pub struct StatsTimelineEventsNode {
    state: StatsTimelineEventsState,
    include_expected_goals: bool,
}

impl StatsTimelineEventsNode {
    pub fn new() -> Self {
        Self {
            state: StatsTimelineEventsState,
            include_expected_goals: false,
        }
    }

    pub fn with_expected_goals(mut self) -> Self {
        self.include_expected_goals = true;
        self
    }

    fn configured_dependencies(&self) -> NodeDependencies {
        let mut dependencies = vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            live_play_dependency(),
            match_stats_dependency(),
            // Keep compact event transfer independent from full partial-sum projection.
            backboard_dependency(),
            ceiling_shot_dependency(),
            wall_aerial_dependency(),
            wall_aerial_shot_dependency(),
            double_tap_dependency(),
            one_timer_dependency(),
            pass_dependency(),
            controlled_play_dependency(),
            fifty_fifty_dependency(),
            kickoff_dependency(),
            possession_dependency(),
            player_possession_dependency(),
            loose_possession_dependency(),
            ball_half_dependency(),
            ball_third_dependency(),
            territorial_pressure_dependency(),
            rotation_dependency(),
            rush_dependency(),
            touch_dependency(),
            whiff_dependency(),
            beaten_to_ball_dependency(),
            wavedash_dependency(),
            flip_impulse_dependency(),
            speed_flip_dependency(),
            half_flip_dependency(),
            flick_dependency(),
            dodge_reset_dependency(),
            ball_carry_dependency(),
            air_dribble_dependency(),
            boost_dependency(),
            bump_dependency(),
            half_volley_dependency(),
            movement_dependency(),
            positioning_dependency(),
            powerslide_dependency(),
            demo_dependency(),
            center_dependency(),
            // The goal-context composition node (which itself pulls match
            // stats plus every goal-tag calculator).
            goal_context_dependency(),
        ];
        if self.include_expected_goals {
            dependencies.push(expected_goals_dependency());
        }
        dependencies
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
        self.configured_dependencies()
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        // Aggregation root only: per-frame work happens in the calculators
        // this node depends on, and every stream's projection lives on its
        // owning node.
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[cfg(test)]
#[path = "stats_timeline_events_tests.rs"]
mod tests;
