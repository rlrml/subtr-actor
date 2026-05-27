use super::*;

impl PassingGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        pass: &PassCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), pass.events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[PassEvent],
    ) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(event) = self.latest_pass_for_goal(events, goal) else {
                continue;
            };
            tags.push(goal_tag_with_modifiers(
                ctx,
                GoalTagKind::PassingGoal,
                1.0,
                mechanic_goal_modifiers(goal, &event.receiver),
                mechanic_goal_evidence(goal, pass_evidence(event)),
            ));
        }
        tags
    }

    fn latest_pass_for_goal<'a>(
        &self,
        events: &'a [PassEvent],
        goal: &GoalContextEvent,
    ) -> Option<&'a PassEvent> {
        events
            .iter()
            .filter(|event| pass_event_matches_goal(event, goal))
            .filter(|event| goal.time - event.time <= self.config.max_pass_to_goal_seconds)
            .max_by(|left, right| {
                left.time
                    .total_cmp(&right.time)
                    .then_with(|| left.frame.cmp(&right.frame))
            })
    }
}
