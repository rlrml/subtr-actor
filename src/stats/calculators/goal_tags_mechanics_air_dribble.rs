use super::*;

impl AirDribbleGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        ball_carry: &BallCarryCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events(), ball_carry.carry_events());
        Ok(())
    }

    pub(super) fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[BallCarryEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_air_dribble_event(goals, events, self.config.max_end_to_goal_seconds)
    }
}
