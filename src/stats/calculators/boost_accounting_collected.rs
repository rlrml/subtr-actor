use super::*;

impl BoostCalculator {
    pub(super) fn apply_pickup_collected_amount(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        is_team_0: bool,
        amount: f32,
        pad_size: Option<BoostPadSize>,
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
        stats.amount_collected += amount;
        team_stats.amount_collected += amount;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(pad_size),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        stats.add_labeled_amount(collected_labels.clone(), amount);
        team_stats.add_labeled_amount(collected_labels.clone(), amount);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels.clone());
        if let Some(pad_size) = pad_size {
            Self::apply_collected_bucket_amount(stats, pad_size, amount);
            Self::apply_collected_bucket_amount(team_stats, pad_size, amount);
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount,
            count: 0,
            labels: collected_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }
}
