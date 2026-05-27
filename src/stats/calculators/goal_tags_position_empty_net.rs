use super::super::*;
use super::*;

impl EmptyNetGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    pub(super) fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(touch) = goal.scorer_last_touch.as_ref() else {
                continue;
            };
            let Some(ball_position) = touch.ball_position else {
                continue;
            };
            let ball = position_to_vec(ball_position);
            let touch_attacking_y = normalized_y(goal.scoring_team_is_team_0, ball);
            if touch_attacking_y > self.config.max_touch_attacking_y {
                continue;
            }

            let player_contexts = if touch.players.is_empty() {
                &goal.players
            } else {
                &touch.players
            };
            let defenders = player_contexts
                .iter()
                .filter(|player| player.is_team_0 != goal.scoring_team_is_team_0)
                .filter_map(|player| {
                    player
                        .position
                        .map(|position| (player, position_to_vec(position)))
                });

            let mut closest_defender_distance = f32::INFINITY;
            let mut smallest_y_margin = f32::INFINITY;
            let mut defender_count = 0;
            let mut evidence = vec![last_touch_evidence(touch), goal_context_evidence(goal)];

            for (defender, position) in defenders {
                defender_count += 1;
                closest_defender_distance = closest_defender_distance.min(position.distance(ball));
                let defender_attacking_y = normalized_y(goal.scoring_team_is_team_0, position);
                let y_margin = touch_attacking_y - defender_attacking_y;
                smallest_y_margin = smallest_y_margin.min(y_margin);
                evidence.push(defender_evidence(defender, goal));
            }

            if defender_count == 0 || smallest_y_margin < self.config.min_defender_y_margin {
                continue;
            }
            if closest_defender_distance < self.config.min_defender_distance {
                continue;
            }

            tags.push(goal_tag(ctx, GoalTagKind::EmptyNetGoal, 1.0, evidence));
        }
        tags
    }
}
