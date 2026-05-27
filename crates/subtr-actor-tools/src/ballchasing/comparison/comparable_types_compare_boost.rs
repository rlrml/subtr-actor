use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};
use super::structs::ComparableBoostStats;

impl ComparableBoostStats {
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

        compare!(bpm, Bpm);
        compare!(avg_amount, AvgAmount);
        compare!(amount_collected, AmountCollected);
        compare!(amount_stolen, AmountStolen);
        compare!(amount_collected_big, AmountCollectedBig);
        compare!(amount_stolen_big, AmountStolenBig);
        compare!(amount_collected_small, AmountCollectedSmall);
        compare!(amount_stolen_small, AmountStolenSmall);
        compare!(count_collected_big, CountCollectedBig);
        compare!(count_stolen_big, CountStolenBig);
        compare!(count_collected_small, CountCollectedSmall);
        compare!(count_stolen_small, CountStolenSmall);
        compare!(amount_overfill, AmountOverfill);
        compare!(amount_overfill_stolen, AmountOverfillStolen);
        compare!(amount_used_while_supersonic, AmountUsedWhileSupersonic);
        compare!(time_zero_boost, TimeZeroBoost);
        compare!(percent_zero_boost, PercentZeroBoost);
        compare!(time_full_boost, TimeFullBoost);
        compare!(percent_full_boost, PercentFullBoost);
        compare!(time_boost_0_25, TimeBoost0To25);
        compare!(time_boost_25_50, TimeBoost25To50);
        compare!(time_boost_50_75, TimeBoost50To75);
        compare!(time_boost_75_100, TimeBoost75To100);
        compare!(percent_boost_0_25, PercentBoost0To25);
        compare!(percent_boost_25_50, PercentBoost25To50);
        compare!(percent_boost_50_75, PercentBoost50To75);
        compare!(percent_boost_75_100, PercentBoost75To100);
    }
}
