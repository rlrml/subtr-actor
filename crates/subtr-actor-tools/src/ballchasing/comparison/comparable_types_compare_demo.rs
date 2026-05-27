use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};
use super::structs::ComparableDemoStats;

impl ComparableDemoStats {
    pub(super) fn compare_player(
        &self,
        scope: &StatScope,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        matcher.compare_field(
            actual.inflicted,
            self.inflicted,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoInflicted,
            },
            config,
        );
        matcher.compare_field(
            actual.taken,
            self.taken,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoTaken,
            },
            config,
        );
    }

    pub(super) fn compare_team(
        &self,
        scope: &StatScope,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        matcher.compare_field(
            actual.inflicted,
            self.inflicted,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoInflicted,
            },
            config,
        );
    }
}
