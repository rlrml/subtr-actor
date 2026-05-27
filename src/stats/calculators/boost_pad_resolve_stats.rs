use super::*;

impl BoostCalculator {
    pub(super) fn apply_resolved_pickup_stats(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
        field_half: BoostPickupFieldHalf,
        stolen: bool,
        amounts: ResolvedPickupAmounts,
    ) {
        stats.amount_collected += amounts.collected_amount_delta;
        team_stats.amount_collected += amounts.collected_amount_delta;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        stats.add_labeled_amount(collected_labels.clone(), amounts.collected_amount_delta);
        team_stats.add_labeled_amount(collected_labels.clone(), amounts.collected_amount_delta);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels);

        Self::apply_resolved_pickup_bucket_amounts(
            stats,
            team_stats,
            pending_pickup,
            pad_size,
            amounts,
        );
        if stolen {
            Self::apply_resolved_stolen_pickup(stats, team_stats, pad_size, field_half, amounts);
        }
        Self::increment_resolved_pickup_counts(stats, team_stats, pad_size, stolen, amounts);
        Self::apply_resolved_pickup_overfill(
            stats, team_stats, pad_size, field_half, stolen, amounts,
        );
    }

    fn apply_resolved_pickup_bucket_amounts(
        stats: &mut BoostStats,
        team_stats: &mut BoostStats,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
        amounts: ResolvedPickupAmounts,
    ) {
        match pending_pickup.pre_applied_pad_size {
            Some(pre_applied_pad_size) if pre_applied_pad_size == pad_size => {
                Self::apply_collected_bucket_amount(
                    stats,
                    pad_size,
                    amounts.collected_amount_delta,
                );
                Self::apply_collected_bucket_amount(
                    team_stats,
                    pad_size,
                    amounts.collected_amount_delta,
                );
            }
            Some(pre_applied_pad_size) => {
                Self::apply_collected_bucket_amount(
                    stats,
                    pre_applied_pad_size,
                    -pending_pickup.pre_applied_collected_amount,
                );
                Self::apply_collected_bucket_amount(
                    team_stats,
                    pre_applied_pad_size,
                    -pending_pickup.pre_applied_collected_amount,
                );
                Self::apply_collected_bucket_amount(stats, pad_size, amounts.collected_amount);
                Self::apply_collected_bucket_amount(team_stats, pad_size, amounts.collected_amount);
            }
            None => {
                Self::apply_collected_bucket_amount(stats, pad_size, amounts.collected_amount);
                Self::apply_collected_bucket_amount(team_stats, pad_size, amounts.collected_amount);
            }
        }
    }
}
