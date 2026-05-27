use super::*;

#[derive(Clone, Copy)]
pub(super) struct BoostLevelTimes {
    tracked_time: f32,
    boost_integral: f32,
    zero: f32,
    hundred: f32,
    boost_0_25: f32,
    boost_25_50: f32,
    boost_50_75: f32,
    boost_75_100: f32,
}

impl BoostLevelTimes {
    pub(super) fn from_interval(dt: f32, previous_boost_amount: f32, boost_amount: f32) -> Self {
        let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
        Self {
            tracked_time: dt,
            boost_integral: average_boost_amount * dt,
            zero: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    0.0,
                    BOOST_ZERO_BAND_RAW,
                ),
            hundred: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    BOOST_FULL_BAND_MIN_RAW,
                    BOOST_MAX_AMOUNT + 1.0,
                ),
            boost_0_25: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    0.0,
                    boost_percent_to_amount(25.0),
                ),
            boost_25_50: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    boost_percent_to_amount(25.0),
                    boost_percent_to_amount(50.0),
                ),
            boost_50_75: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    boost_percent_to_amount(50.0),
                    boost_percent_to_amount(75.0),
                ),
            boost_75_100: dt
                * boost_range_fraction(
                    previous_boost_amount,
                    boost_amount,
                    boost_percent_to_amount(75.0),
                    BOOST_MAX_AMOUNT + 1.0,
                ),
        }
    }

    pub(super) fn apply(self, stats: &mut BoostStats) {
        stats.tracked_time += self.tracked_time;
        stats.boost_integral += self.boost_integral;
        stats.time_zero_boost += self.zero;
        stats.time_hundred_boost += self.hundred;
        stats.time_boost_0_25 += self.boost_0_25;
        stats.time_boost_25_50 += self.boost_25_50;
        stats.time_boost_50_75 += self.boost_50_75;
        stats.time_boost_75_100 += self.boost_75_100;
    }
}

fn boost_range_fraction(start: f32, end: f32, min: f32, max: f32) -> f32 {
    BoostCalculator::interval_fraction_in_boost_range(start, end, min, max)
}
