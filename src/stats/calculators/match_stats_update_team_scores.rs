use super::*;

impl MatchStatsCalculator {
    pub(super) fn update_team_score_contexts(
        &mut self,
        gameplay: &GameplayState,
        players: &PlayerFrameState,
    ) {
        let (Some(team_zero_score), Some(team_one_score)) =
            (gameplay.team_zero_score, gameplay.team_one_score)
        else {
            return;
        };

        if let Some((prev_team_zero_score, prev_team_one_score)) = self.previous_team_scores {
            self.record_goals_against_last_defenders(
                players,
                team_zero_score - prev_team_zero_score,
                team_one_score - prev_team_one_score,
            );
        }

        self.previous_team_scores = Some((team_zero_score, team_one_score));
    }

    fn record_goals_against_last_defenders(
        &mut self,
        players: &PlayerFrameState,
        team_zero_delta: i32,
        team_one_delta: i32,
    ) {
        if team_zero_delta > 0 {
            self.record_last_defender_goal_conceded(players, false, team_zero_delta);
        }
        if team_one_delta > 0 {
            self.record_last_defender_goal_conceded(players, true, team_one_delta);
        }
    }

    fn record_last_defender_goal_conceded(
        &mut self,
        players: &PlayerFrameState,
        is_team_0: bool,
        delta: i32,
    ) {
        if let Some(last_defender) = self.last_defender(players, is_team_0) {
            if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                stats.scoring_context.goals_conceded_while_last_defender += delta as u32;
            }
        }
    }
}
