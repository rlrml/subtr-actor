use super::*;

impl BoostCalculator {
    pub(super) fn apply_resolved_stolen_pickup(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        amounts: ResolvedPickupAmounts,
    ) {
        stats.amount_stolen += amounts.collected_amount;
        team_stats.amount_stolen += amounts.collected_amount;
        let stolen_labels = [
            boost_transaction_label("stolen"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        stats.add_labeled_amount(stolen_labels.clone(), amounts.collected_amount);
        team_stats.add_labeled_amount(stolen_labels, amounts.collected_amount);
    }

    pub(super) fn increment_resolved_pickup_counts(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pad_size: BoostPadSize,
        stolen: bool,
        amounts: ResolvedPickupAmounts,
    ) {
        match pad_size {
            BoostPadSize::Big => {
                stats.big_pads_collected += 1;
                team_stats.big_pads_collected += 1;
                if stolen {
                    stats.big_pads_stolen += 1;
                    team_stats.big_pads_stolen += 1;
                    stats.amount_stolen_big += amounts.collected_amount;
                    team_stats.amount_stolen_big += amounts.collected_amount;
                }
            }
            BoostPadSize::Small => {
                stats.small_pads_collected += 1;
                team_stats.small_pads_collected += 1;
                if stolen {
                    stats.small_pads_stolen += 1;
                    team_stats.small_pads_stolen += 1;
                    stats.amount_stolen_small += amounts.collected_amount;
                    team_stats.amount_stolen_small += amounts.collected_amount;
                }
            }
        }
    }

    pub(super) fn apply_resolved_pickup_overfill(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        stolen: bool,
        amounts: ResolvedPickupAmounts,
    ) {
        stats.overfill_total += amounts.overfill;
        team_stats.overfill_total += amounts.overfill;
        let overfill_labels = [
            boost_transaction_label("overfill"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        stats.add_labeled_amount(overfill_labels.clone(), amounts.overfill);
        team_stats.add_labeled_amount(overfill_labels, amounts.overfill);
        if stolen {
            stats.overfill_from_stolen += amounts.overfill;
            team_stats.overfill_from_stolen += amounts.overfill;
        }
    }
}
