use super::*;

impl BoostCalculator {
    pub(super) fn apply_collected_bucket_amount(
        stats: &mut BoostStats,
        pad_size: BoostPadSize,
        amount: f32,
    ) {
        if amount == 0.0 {
            return;
        }

        match pad_size {
            BoostPadSize::Big => stats.amount_collected_big += amount,
            BoostPadSize::Small => stats.amount_collected_small += amount,
        }
    }

    pub(super) fn interval_fraction_in_boost_range(
        start_boost: f32,
        end_boost: f32,
        min_boost: f32,
        max_boost: f32,
    ) -> f32 {
        if (end_boost - start_boost).abs() <= f32::EPSILON {
            return ((start_boost >= min_boost) && (start_boost < max_boost)) as i32 as f32;
        }

        let t_at_min = (min_boost - start_boost) / (end_boost - start_boost);
        let t_at_max = (max_boost - start_boost) / (end_boost - start_boost);
        let interval_start = t_at_min.min(t_at_max).max(0.0);
        let interval_end = t_at_min.max(t_at_max).min(1.0);
        (interval_end - interval_start).max(0.0)
    }
}
