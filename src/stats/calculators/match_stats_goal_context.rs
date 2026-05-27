use super::*;

impl MatchStatsCalculator {
    pub(super) fn goal_player_contexts(
        &self,
        players: &PlayerFrameState,
        scoring_team_is_team_0: bool,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
    ) -> Vec<GoalPlayerContext> {
        players
            .players
            .iter()
            .map(|player| {
                let most_back_player = if player.is_team_0 == scoring_team_is_team_0 {
                    scoring_team_most_back_player
                } else {
                    defending_team_most_back_player
                };
                let boost_leadup = self.boost_leadup_for_player(&player.player_id);
                GoalPlayerContext {
                    player: player.player_id.clone(),
                    is_team_0: player.is_team_0,
                    position: player.position().map(GoalContextPosition::from),
                    boost_amount: player.boost_amount.or(player.last_boost_amount),
                    average_boost_in_leadup: boost_leadup.map(|stats| stats.average_boost),
                    min_boost_in_leadup: boost_leadup.map(|stats| stats.min_boost),
                    is_most_back: most_back_player == Some(&player.player_id),
                }
            })
            .collect()
    }

    pub(super) fn record_goal_context_stats(
        &mut self,
        players: &PlayerFrameState,
        goal_event: &GoalEvent,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
    ) {
        self.record_most_back_goal_stats(
            scoring_team_most_back_player,
            defending_team_most_back_player,
        );

        for player in players
            .players
            .iter()
            .filter(|player| player.is_team_0 != goal_event.scoring_team_is_team_0)
        {
            let boost_leadup = self.boost_leadup_for_player(&player.player_id);
            self.player_stats
                .entry(player.player_id.clone())
                .or_default()
                .scoring_context
                .record_goal_against_snapshot(
                    player.boost_amount.or(player.last_boost_amount),
                    player.position().map(GoalContextPosition::from),
                    boost_leadup,
                );
        }
    }

    fn record_most_back_goal_stats(
        &mut self,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
    ) {
        if let Some(player_id) = scoring_team_most_back_player {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .scoring_context
                .goals_for_while_most_back += 1;
        }

        if let Some(player_id) = defending_team_most_back_player {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .scoring_context
                .goals_against_while_most_back += 1;
        }
    }
}
