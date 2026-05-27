use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_used_allocation::allocate_used_boost;

impl BoostCalculator {
    pub(super) fn update_player_used_totals(
        &mut self,
        player: &PlayerSample,
        boost_amount: f32,
        frame: &FrameInfo,
        vertical_state: &PlayerVerticalState,
        context: &BoostUpdateContext,
    ) -> f32 {
        let boost_before = self
            .previous_boost_amounts
            .get(&player.player_id)
            .copied()
            .or(player.last_boost_amount);
        let stats = self
            .player_stats
            .entry(player.player_id.clone())
            .or_default();
        let previous_amount_used = stats.amount_used;
        let demo_reset_boost_amount = self
            .demo_reset_boost_amounts
            .get(&player.player_id)
            .copied()
            .unwrap_or(0.0);
        let amount_used_raw =
            (stats.amount_obtained() - demo_reset_boost_amount - boost_amount).max(0.0);
        let amount_used = amount_used_raw.max(stats.amount_used);
        let used_ledger_event = if context.track_boost_levels {
            allocate_used_boost(
                player,
                stats,
                team_stats_for_player(
                    player.is_team_0,
                    &mut self.team_zero_stats,
                    &mut self.team_one_stats,
                ),
                amount_used,
                boost_amount,
                boost_before,
                frame,
                vertical_state,
                self.previous_player_speeds.get(&player.player_id).copied(),
                context,
            )
        } else {
            None
        };
        stats.amount_used = amount_used;
        if let Some(event) = used_ledger_event {
            self.record_ledger_event(event);
        }
        amount_used - previous_amount_used
    }
}

fn team_stats_for_player<'a>(
    is_team_0: bool,
    team_zero_stats: &'a mut BoostStats,
    team_one_stats: &'a mut BoostStats,
) -> &'a mut BoostStats {
    if is_team_0 {
        team_zero_stats
    } else {
        team_one_stats
    }
}
