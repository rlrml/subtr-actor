use super::*;

impl FlipResetGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        dodge_reset: &DodgeResetCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(
            match_stats.goal_context_events(),
            dodge_reset.confirmed_flip_reset_events(),
        );
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[ConfirmedFlipResetEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::FlipResetGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}
