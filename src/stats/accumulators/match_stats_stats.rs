use super::*;

#[derive(Debug, Clone, Default)]
pub struct CoreStatsAccumulator {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
}

impl CoreStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        &self.player_stats
    }

    pub fn player_stats_for(&self, player_id: &PlayerId) -> CorePlayerStats {
        self.player_stats
            .get(player_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn ensure_player(&mut self, player_id: PlayerId, is_team_0: bool) {
        self.player_teams.insert(player_id.clone(), is_team_0);
        self.player_stats.entry(player_id).or_default();
    }

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    pub fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
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
                stats
                    .scoring_context
                    .goal_after_kickoff
                    .merge(&player_stats.scoring_context.goal_after_kickoff);
                stats
                    .scoring_context
                    .goal_buildup
                    .merge(&player_stats.scoring_context.goal_buildup);
                stats
                    .scoring_context
                    .goal_ball_air_time
                    .merge(&player_stats.scoring_context.goal_ball_air_time);
                stats
            },
        );
        stats
            .scoring_context
            .goal_after_kickoff
            .goal_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
    }

    pub fn apply_player_event(&mut self, event: &CorePlayerStatsEvent) {
        self.player_teams
            .insert(event.player.clone(), event.is_team_0);
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        apply_core_player_delta(stats, &event.delta);
    }
}

fn apply_core_player_delta(stats: &mut CorePlayerStats, delta: &CorePlayerStats) {
    stats.score += delta.score;
    stats.goals += delta.goals;
    stats.assists += delta.assists;
    stats.saves += delta.saves;
    stats.shots += delta.shots;
    apply_player_scoring_context_delta(&mut stats.scoring_context, &delta.scoring_context);
}

fn apply_player_scoring_context_delta(
    stats: &mut PlayerScoringContextStats,
    delta: &PlayerScoringContextStats,
) {
    stats.goals_conceded_while_last_defender += delta.goals_conceded_while_last_defender;
    stats.goals_for_while_most_back += delta.goals_for_while_most_back;
    stats.goals_against_while_most_back += delta.goals_against_while_most_back;
    stats.goal_against_boost_sample_count += delta.goal_against_boost_sample_count;
    stats.cumulative_boost_on_goals_against += delta.cumulative_boost_on_goals_against;
    if delta.last_boost_on_goal_against.is_some() {
        stats.last_boost_on_goal_against = delta.last_boost_on_goal_against;
    }
    stats.goal_against_boost_leadup_sample_count += delta.goal_against_boost_leadup_sample_count;
    stats.cumulative_average_boost_in_goal_against_leadup +=
        delta.cumulative_average_boost_in_goal_against_leadup;
    stats.cumulative_min_boost_in_goal_against_leadup +=
        delta.cumulative_min_boost_in_goal_against_leadup;
    if delta.last_average_boost_in_goal_against_leadup.is_some() {
        stats.last_average_boost_in_goal_against_leadup =
            delta.last_average_boost_in_goal_against_leadup;
    }
    if delta.last_min_boost_in_goal_against_leadup.is_some() {
        stats.last_min_boost_in_goal_against_leadup = delta.last_min_boost_in_goal_against_leadup;
    }
    stats.goal_against_position_sample_count += delta.goal_against_position_sample_count;
    stats.cumulative_goal_against_position_x += delta.cumulative_goal_against_position_x;
    stats.cumulative_goal_against_position_y += delta.cumulative_goal_against_position_y;
    stats.cumulative_goal_against_position_z += delta.cumulative_goal_against_position_z;
    if delta.last_goal_against_position.is_some() {
        stats.last_goal_against_position = delta.last_goal_against_position;
    }
    stats.scoring_goal_last_touch_position_sample_count +=
        delta.scoring_goal_last_touch_position_sample_count;
    stats.cumulative_scoring_goal_last_touch_position_x +=
        delta.cumulative_scoring_goal_last_touch_position_x;
    stats.cumulative_scoring_goal_last_touch_position_y +=
        delta.cumulative_scoring_goal_last_touch_position_y;
    stats.cumulative_scoring_goal_last_touch_position_z +=
        delta.cumulative_scoring_goal_last_touch_position_z;
    if delta.last_scoring_goal_last_touch_position.is_some() {
        stats.last_scoring_goal_last_touch_position = delta.last_scoring_goal_last_touch_position;
    }
    stats.goal_after_kickoff.merge(&delta.goal_after_kickoff);
    stats.goal_buildup.merge(&delta.goal_buildup);
    stats.goal_ball_air_time.merge(&delta.goal_ball_air_time);
}
