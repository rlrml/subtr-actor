use super::*;

impl FlickGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        flick: &FlickCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), flick.events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[FlickEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::FlickGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl OneTimerGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        one_timer: &OneTimerCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), one_timer.events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[OneTimerEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::OneTimerGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl DoubleTapGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        double_tap: &DoubleTapCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), double_tap.events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[DoubleTapEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::DoubleTapGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}
