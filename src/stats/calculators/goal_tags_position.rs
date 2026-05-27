use super::super::*;
use super::*;

impl AerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_height(goals, GoalTagKind::AerialGoal, self.config.min_ball_z)
    }
}

impl HighAerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_height(goals, GoalTagKind::HighAerialGoal, self.config.min_ball_z)
    }
}

impl LongDistanceGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_attacking_y(
            goals,
            GoalTagKind::LongDistanceGoal,
            self.config.max_attacking_y,
        )
    }
}

impl OwnHalfGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_recent_attacking_y(
            goals,
            GoalTagKind::OwnHalfGoal,
            self.config.max_attacking_y,
            OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
        )
    }
}

impl CounterAttackGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| goal.goal_buildup == GoalBuildupKind::CounterAttack)
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index, goal },
                    GoalTagKind::CounterAttackGoal,
                    1.0,
                    vec![goal_buildup_evidence(goal), goal_context_evidence(goal)],
                )
            })
            .collect()
    }
}
