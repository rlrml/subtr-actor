use super::*;

impl BoostCalculator {
    pub(super) fn apply_inactive_pickup(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        is_team_0: bool,
        amount: f32,
        pad_size: BoostPadSize,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.amount_collected_inactive += amount;
        team_stats.amount_collected_inactive += amount;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Inactive),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        stats.add_labeled_amount(collected_labels.clone(), amount);
        team_stats.add_labeled_amount(collected_labels.clone(), amount);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels.clone());
        Self::increment_inactive_pad_count(stats, team_stats, pad_size);
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount,
            count: 1,
            labels: collected_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }

    fn increment_inactive_pad_count(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pad_size: BoostPadSize,
    ) {
        match pad_size {
            BoostPadSize::Big => {
                stats.big_pads_collected_inactive += 1;
                team_stats.big_pads_collected_inactive += 1;
            }
            BoostPadSize::Small => {
                stats.small_pads_collected_inactive += 1;
                team_stats.small_pads_collected_inactive += 1;
            }
        }
    }
}
