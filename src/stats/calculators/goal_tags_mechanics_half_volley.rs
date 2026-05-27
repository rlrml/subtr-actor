use super::*;

impl HalfVolleyGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        half_volley: &HalfVolleyCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), half_volley.events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        half_volley_events: &[HalfVolleyEvent],
    ) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(candidate) = self.tag_goals_by_half_volley_event(goal, half_volley_events)
            else {
                continue;
            };
            tags.push(goal_tag_with_modifiers(
                ctx,
                GoalTagKind::HalfVolleyGoal,
                1.0,
                mechanic_goal_modifiers(goal, &candidate.player),
                mechanic_goal_evidence(goal, half_volley_evidence(candidate)),
            ));
        }
        tags
    }

    fn tag_goals_by_half_volley_event<'a>(
        &self,
        goal: &GoalContextEvent,
        half_volley_events: &'a [HalfVolleyEvent],
    ) -> Option<&'a HalfVolleyEvent> {
        half_volley_events
            .iter()
            .filter(|candidate| self.candidate_matches_goal(candidate, goal))
            .max_by(|left, right| {
                left.time
                    .total_cmp(&right.time)
                    .then_with(|| left.frame.cmp(&right.frame))
            })
    }

    fn candidate_matches_goal(&self, candidate: &HalfVolleyEvent, goal: &GoalContextEvent) -> bool {
        const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

        if candidate.is_team_0 != goal.scoring_team_is_team_0
            || candidate.time > goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
            || candidate.frame > goal.frame
            || goal.time - candidate.time > self.config.max_touch_to_goal_seconds
            || candidate.goal_alignment < self.config.min_goal_alignment
        {
            return false;
        }

        goal.scorer_last_touch
            .as_ref()
            .is_some_and(|touch| touch.player == candidate.player && touch.frame == candidate.frame)
    }
}
