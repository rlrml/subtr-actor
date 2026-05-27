use super::*;

impl BoostCalculator {
    pub(super) fn record_resolved_pickup_ledger_events(
        &mut self,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        stolen: bool,
        amounts: ResolvedPickupAmounts,
    ) {
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            is_team_0: pending_pickup.is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount: amounts.collected_amount_delta,
            count: 1,
            labels: collected_labels.into_iter().collect(),
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        });
        if stolen {
            self.record_resolved_stolen_pickup_ledger_event(
                pending_pickup,
                pad_size,
                field_half,
                amounts,
            );
        }
        self.record_resolved_overfill_ledger_event(pending_pickup, pad_size, field_half, amounts);
    }

    fn record_resolved_stolen_pickup_ledger_event(
        &mut self,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        amounts: ResolvedPickupAmounts,
    ) {
        let stolen_labels = [
            boost_transaction_label("stolen"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            is_team_0: pending_pickup.is_team_0,
            transaction: BoostLedgerTransactionKind::Stolen,
            amount: amounts.collected_amount,
            count: 1,
            labels: stolen_labels.into_iter().collect(),
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        });
    }

    fn record_resolved_overfill_ledger_event(
        &mut self,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        amounts: ResolvedPickupAmounts,
    ) {
        let overfill_labels = [
            boost_transaction_label("overfill"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            is_team_0: pending_pickup.is_team_0,
            transaction: BoostLedgerTransactionKind::Overfill,
            amount: amounts.overfill,
            count: 0,
            labels: overfill_labels.into_iter().collect(),
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        });
    }
}
