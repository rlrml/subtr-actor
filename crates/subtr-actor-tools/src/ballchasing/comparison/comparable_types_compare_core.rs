use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};
use super::structs::ComparableCoreStats;

impl ComparableCoreStats {
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

        compare!(score, Score);
        compare!(goals, Goals);
        compare!(assists, Assists);
        compare!(saves, Saves);
        compare!(shots, Shots);
        compare!(shooting_percentage, ShootingPercentage);
    }
}
