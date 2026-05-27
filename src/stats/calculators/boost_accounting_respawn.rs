use super::*;

impl BoostCalculator {
    pub(super) fn apply_respawn_amount(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        is_team_0: bool,
        amount: f32,
    ) {
        if amount <= 0.0 {
            return;
        }

        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.amount_respawned += amount;
        team_stats.amount_respawned += amount;
        let respawn_labels = [boost_transaction_label("respawn")];
        stats.add_labeled_amount(respawn_labels.clone(), amount);
        team_stats.add_labeled_amount(respawn_labels.clone(), amount);
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            is_team_0,
            transaction: BoostLedgerTransactionKind::Respawn,
            amount,
            count: 0,
            labels: respawn_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }
}
