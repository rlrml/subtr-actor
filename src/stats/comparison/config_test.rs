use super::super::model::TeamColor;
use super::*;

#[test]
fn test_match_config_defaults_to_exact() {
    let config = MatchConfig::exact().with_rule(
        "time zero boost abs<=1",
        |target| target.key == StatKey::TimeZeroBoost,
        approx_abs(1.0),
    );

    let default_target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Boost,
        key: StatKey::CountCollectedBig,
    };
    let tolerant_target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Boost,
        key: StatKey::TimeZeroBoost,
    };

    assert!(!config.evaluate(3.0, 2.0, &default_target).matches);
    assert!(config.evaluate(3.5, 3.0, &tolerant_target).matches);
}

#[test]
fn test_match_config_uses_last_matching_rule() {
    let config = MatchConfig::exact()
        .with_rule(
            "all movement abs<=1",
            |target| target.domain == StatDomain::Movement,
            approx_abs(1.0),
        )
        .with_rule(
            "movement total distance abs<=10",
            |target| target.key == StatKey::TotalDistance,
            approx_abs(10.0),
        );

    let target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Movement,
        key: StatKey::TotalDistance,
    };

    let outcome = config.evaluate(1008.0, 1000.0, &target);
    assert!(outcome.matches);
    assert_eq!(outcome.description, "movement total distance abs<=10");
}
