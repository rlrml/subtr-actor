use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};
use super::structs::ComparableMovementStats;

impl ComparableMovementStats {
    pub(super) fn compare(
        &self,
        scope: &StatScope,
        domain: StatDomain,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        macro_rules! compare {
            ($field:ident, $key:ident) => {
                matcher.compare_field(
                    actual.$field,
                    self.$field,
                    ComparisonTarget {
                        scope: scope.clone(),
                        domain,
                        key: StatKey::$key,
                    },
                    config,
                );
            };
        }

        compare!(avg_speed, AvgSpeed);
        compare!(total_distance, TotalDistance);
        compare!(time_supersonic_speed, TimeSupersonicSpeed);
        compare!(time_boost_speed, TimeBoostSpeed);
        compare!(time_slow_speed, TimeSlowSpeed);
        compare!(time_ground, TimeGround);
        compare!(time_low_air, TimeLowAir);
        compare!(time_high_air, TimeHighAir);
        compare!(time_powerslide, TimePowerslide);
        compare!(count_powerslide, CountPowerslide);
        compare!(avg_powerslide_duration, AvgPowerslideDuration);
        compare!(avg_speed_percentage, AvgSpeedPercentage);
        compare!(percent_slow_speed, PercentSlowSpeed);
        compare!(percent_boost_speed, PercentBoostSpeed);
        compare!(percent_supersonic_speed, PercentSupersonicSpeed);
        compare!(percent_ground, PercentGround);
        compare!(percent_low_air, PercentLowAir);
        compare!(percent_high_air, PercentHighAir);
    }
}
