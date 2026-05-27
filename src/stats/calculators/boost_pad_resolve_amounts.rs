use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct ResolvedPickupAmounts {
    pub(super) collected_amount: f32,
    pub(super) collected_amount_delta: f32,
    pub(super) overfill: f32,
}

impl ResolvedPickupAmounts {
    pub(super) fn new(pending_pickup: &PendingBoostPickup, pad_size: BoostPadSize) -> Self {
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let collected_amount = (BOOST_MAX_AMOUNT - pending_pickup.previous_boost_amount)
            .min(nominal_gain)
            .max(pending_pickup.pre_applied_collected_amount);
        Self {
            collected_amount,
            collected_amount_delta: collected_amount - pending_pickup.pre_applied_collected_amount,
            overfill: (nominal_gain - collected_amount).max(0.0),
        }
    }
}
