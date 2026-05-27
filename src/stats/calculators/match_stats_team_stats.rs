use super::match_stats_delta::player_id_sort_key;
use super::*;

impl MatchStatsCalculator {
    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    pub(super) fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
        let mut player_stats: Vec<_> = self
            .player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .collect();
        player_stats.sort_by_cached_key(|(player_id, _)| player_id_sort_key(player_id));

        let mut stats = player_stats.into_iter().fold(
            CoreTeamStats::default(),
            |mut stats, (_, player_stats)| {
                stats.score += player_stats.score;
                stats.goals += player_stats.goals;
                stats.assists += player_stats.assists;
                stats.saves += player_stats.saves;
                stats.shots += player_stats.shots;
                merge_scoring_context(&mut stats.scoring_context, &player_stats.scoring_context);
                stats
            },
        );
        sort_goal_context_stats(&mut stats.scoring_context);
        stats
    }
}

fn merge_scoring_context(team: &mut TeamScoringContextStats, player: &PlayerScoringContextStats) {
    team.goal_after_kickoff.merge(&player.goal_after_kickoff);
    team.goal_buildup.merge(&player.goal_buildup);
    team.goal_ball_air_time.merge(&player.goal_ball_air_time);
}

fn sort_goal_context_stats(stats: &mut TeamScoringContextStats) {
    stats
        .goal_after_kickoff
        .goal_times
        .sort_by(|left, right| left.total_cmp(right));
    stats
        .goal_ball_air_time
        .goal_ball_air_times
        .sort_by(|left, right| left.total_cmp(right));
}
