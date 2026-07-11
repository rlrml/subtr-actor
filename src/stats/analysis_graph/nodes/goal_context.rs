use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Marker state for the goal-context composition node; the projected events
/// live in the graph's transaction log like every other stream's.
#[derive(Debug, Clone, Default)]
pub struct GoalContextState;

/// Composes the `goal_context` event stream from [`MatchStatsCalculator`]'s
/// per-goal context events and the combined goal-tag assignments of every
/// goal-tag calculator.
///
/// Goal context has no single owning calculator: the base events come from
/// match stats, while the tag set is the union of the 19 goal-tag
/// calculators' assignments, combined *at projection time* (see
/// [`combined_goal_tag_assignments`]). In the node-owned projection
/// architecture such compositions are nodes — this node declares the inputs
/// as dependencies and reads them from the [`AnalysisStateContext`], holding
/// no per-frame state of its own.
pub struct GoalContextNode {
    state: GoalContextState,
}

impl GoalContextNode {
    pub fn new() -> Self {
        Self {
            state: GoalContextState,
        }
    }
}

impl Default for GoalContextNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for GoalContextNode {
    type State = GoalContextState;

    fn name(&self) -> &'static str {
        "goal_context"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::GOAL_CONTEXT_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            match_stats_dependency(),
            aerial_goal_dependency(),
            high_aerial_goal_dependency(),
            long_distance_goal_dependency(),
            own_half_goal_dependency(),
            empty_net_goal_dependency(),
            counter_attack_goal_dependency(),
            sustained_pressure_goal_dependency(),
            flick_goal_dependency(),
            ceiling_shot_goal_dependency(),
            double_tap_goal_dependency(),
            one_timer_goal_dependency(),
            passing_goal_dependency(),
            air_dribble_goal_dependency(),
            flip_reset_goal_dependency(),
            flip_into_ball_goal_dependency(),
            bump_goal_dependency(),
            demo_goal_dependency(),
            half_volley_goal_dependency(),
            kickoff_goal_dependency(),
        ]
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        // Pure composition: the per-frame work happens in the calculators this
        // node depends on; everything here is done at projection time.
        Ok(())
    }

    fn project_events(&self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        let goal_tag_assignments = combined_goal_tag_assignments(&[
            ctx.get::<AerialGoalCalculator>()?.events(),
            ctx.get::<HighAerialGoalCalculator>()?.events(),
            ctx.get::<LongDistanceGoalCalculator>()?.events(),
            ctx.get::<OwnHalfGoalCalculator>()?.events(),
            ctx.get::<EmptyNetGoalCalculator>()?.events(),
            ctx.get::<CounterAttackGoalCalculator>()?.events(),
            ctx.get::<SustainedPressureGoalCalculator>()?.events(),
            ctx.get::<FlickGoalCalculator>()?.events(),
            ctx.get::<CeilingShotGoalCalculator>()?.events(),
            ctx.get::<DoubleTapGoalCalculator>()?.events(),
            ctx.get::<OneTimerGoalCalculator>()?.events(),
            ctx.get::<PassingGoalCalculator>()?.events(),
            ctx.get::<AirDribbleGoalCalculator>()?.events(),
            ctx.get::<FlipResetGoalCalculator>()?.events(),
            ctx.get::<FlipIntoBallGoalCalculator>()?.events(),
            ctx.get::<BumpGoalCalculator>()?.events(),
            ctx.get::<DemoGoalCalculator>()?.events(),
            ctx.get::<HalfVolleyGoalCalculator>()?.events(),
            ctx.get::<KickoffGoalCalculator>()?.events(),
        ]);
        let goal_context =
            goal_context_events_with_tags(match_stats.goal_context_events(), &goal_tag_assignments);

        let mut assembler = EventAssembler::new();
        // Goal context is enriched after the goal frame: the scorer is reconciled
        // once the scoreboard attributes the goal, the goal-tag set is combined
        // from the goal calculators at projection time, and pressure durations
        // attach at finish — so an interim goal-context event stays Confirmed.
        for event in &goal_context {
            assembler.push(
                "goal_context",
                event.frame,
                EventLifecycle::Confirmed,
                moment(event.frame, event.time),
                EventPayload::GoalContext(event.clone()),
                event.scorer.clone(),
                None,
                Some(event.scoring_team_is_team_0),
                None,
                event
                    .ball_position
                    .map(|position| [position.x, position.y, position.z]),
                None,
            );
        }
        Ok(assembler.into_events())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(GoalContextNode::new())
}
