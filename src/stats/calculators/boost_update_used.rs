use super::*;
use boost_update_context::BoostUpdateContext;

impl BoostCalculator {
    pub(super) fn update_used_boost(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        context: &BoostUpdateContext,
    ) {
        let mut team_zero_used = self.team_zero_stats.amount_used;
        let mut team_one_used = self.team_one_stats.amount_used;
        for player in &players.players {
            let Some(amount_used_delta) =
                self.update_player_used_boost(frame, player, vertical_state, context)
            else {
                continue;
            };
            if player.is_team_0 {
                team_zero_used += amount_used_delta;
            } else {
                team_one_used += amount_used_delta;
            }
        }
        self.team_zero_stats.amount_used = team_zero_used;
        self.team_one_stats.amount_used = team_one_used;
    }

    fn update_player_used_boost(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        vertical_state: &PlayerVerticalState,
        context: &BoostUpdateContext,
    ) -> Option<f32> {
        if self.pending_demo_respawns.contains_key(&player.player_id) {
            return None;
        }
        let boost_amount = player.boost_amount?;
        let boost_before = self
            .previous_boost_amounts
            .get(&player.player_id)
            .copied()
            .or(player.last_boost_amount);
        let amount_used_delta =
            self.update_player_used_totals(player, boost_amount, frame, vertical_state, context);
        if amount_used_delta <= 0.0 {
            return None;
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: frame.frame_number,
            time: frame.time,
            player_id: player.player_id.clone(),
            is_team_0: player.is_team_0,
            transaction: BoostLedgerTransactionKind::Used,
            amount: amount_used_delta,
            count: 0,
            labels: [boost_transaction_label("used")].into_iter().collect(),
            boost_before,
            boost_after: Some(boost_amount),
        });
        Some(amount_used_delta)
    }
}
