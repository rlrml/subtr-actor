use super::*;

pub const BOOST_INVARIANT_BASE_TOLERANCE_RAW: f32 = 2.0;
pub const BOOST_INVARIANT_PER_PICKUP_TOLERANCE_RAW: f32 = 0.3;

pub fn nominal_pickup_amount_from_counts(stats: &BoostStats) -> f32 {
    stats.big_pads_collected as f32 * BOOST_MAX_AMOUNT
        + stats.small_pads_collected as f32 * boost_percent_to_amount(12.0)
}

pub fn nominal_stolen_pickup_amount_from_counts(stats: &BoostStats) -> f32 {
    stats.big_pads_stolen as f32 * BOOST_MAX_AMOUNT
        + stats.small_pads_stolen as f32 * boost_percent_to_amount(12.0)
}

pub(super) fn nominal_pickup_tolerance(pickup_count: u32) -> f32 {
    BOOST_INVARIANT_BASE_TOLERANCE_RAW
        + BOOST_INVARIANT_PER_PICKUP_TOLERANCE_RAW * pickup_count as f32
}
