use super::super::model::{StatDomain, StatKey};
use super::match_config::{approx_abs, MatchConfig};
use super::recommended_predicates::{
    boost_amount_key, boost_timing_key, movement_distance_key, movement_distance_predicate,
    movement_timing_key, positioning_predicate,
};

pub fn recommended_match_config() -> MatchConfig {
    MatchConfig::exact()
        .with_rule(
            "shooting percentage abs<=0.01",
            |target| target.key == StatKey::ShootingPercentage,
            approx_abs(0.01),
        )
        .with_rule(
            "boost amount style fields abs<=2",
            boost_amount_key,
            approx_abs(2.0),
        )
        .with_rule(
            "boost timing and percentage fields abs<=1",
            boost_timing_key,
            approx_abs(1.0),
        )
        .with_rule(
            "movement timing and percentage fields abs<=1",
            movement_timing_key,
            approx_abs(1.0),
        )
        .with_rule(
            "movement distance/speed fields tolerate external rounding",
            movement_distance_key,
            movement_distance_predicate,
        )
        .with_rule(
            "positioning fields abs<=1 or 50 depending on metric",
            |target| target.domain == StatDomain::Positioning,
            positioning_predicate,
        )
}
