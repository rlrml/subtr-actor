use super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};

type MatchSelector = dyn Fn(&ComparisonTarget) -> bool;
type MatchPredicate = dyn Fn(f64, f64, &ComparisonTarget) -> bool;

struct MatchRule {
    description: String,
    selector: Box<MatchSelector>,
    predicate: Box<MatchPredicate>,
}

#[derive(Default)]
pub struct MatchConfig {
    rules: Vec<MatchRule>,
}

struct MatchOutcome<'a> {
    matches: bool,
    description: &'a str,
}

impl MatchConfig {
    fn exact() -> Self {
        Self::default()
    }

    fn with_rule<S, P>(mut self, description: impl Into<String>, selector: S, predicate: P) -> Self
    where
        S: Fn(&ComparisonTarget) -> bool + 'static,
        P: Fn(f64, f64, &ComparisonTarget) -> bool + 'static,
    {
        self.rules.push(MatchRule {
            description: description.into(),
            selector: Box::new(selector),
            predicate: Box::new(predicate),
        });
        self
    }

    fn evaluate<'a>(
        &'a self,
        actual: f64,
        expected: f64,
        target: &ComparisonTarget,
    ) -> MatchOutcome<'a> {
        let default = MatchOutcome {
            matches: actual == expected,
            description: "exact",
        };

        self.rules
            .iter()
            .rev()
            .find(|rule| (rule.selector)(target))
            .map(|rule| MatchOutcome {
                matches: (rule.predicate)(actual, expected, target),
                description: &rule.description,
            })
            .unwrap_or(default)
    }
}

pub(super) fn approx_abs(abs_tol: f64) -> impl Fn(f64, f64, &ComparisonTarget) -> bool {
    move |actual, expected, _| (actual - expected).abs() <= abs_tol
}

pub fn recommended_match_config() -> MatchConfig {
    MatchConfig::exact()
        .with_rule(
            "shooting percentage abs<=0.01",
            |target| target.key == StatKey::ShootingPercentage,
            approx_abs(0.01),
        )
        .with_rule(
            "boost amount style fields abs<=2",
            |target| {
                matches!(
                    target.key,
                    StatKey::AmountCollected
                        | StatKey::AmountStolen
                        | StatKey::AmountCollectedBig
                        | StatKey::AmountStolenBig
                        | StatKey::AmountCollectedSmall
                        | StatKey::AmountStolenSmall
                        | StatKey::AmountOverfill
                        | StatKey::AmountOverfillStolen
                        | StatKey::AmountUsedWhileSupersonic
                )
            },
            approx_abs(2.0),
        )
        .with_rule(
            "boost timing and percentage fields abs<=1",
            |target| {
                matches!(
                    target.key,
                    StatKey::Bpm
                        | StatKey::AvgAmount
                        | StatKey::TimeZeroBoost
                        | StatKey::PercentZeroBoost
                        | StatKey::TimeFullBoost
                        | StatKey::PercentFullBoost
                        | StatKey::TimeBoost0To25
                        | StatKey::TimeBoost25To50
                        | StatKey::TimeBoost50To75
                        | StatKey::TimeBoost75To100
                        | StatKey::PercentBoost0To25
                        | StatKey::PercentBoost25To50
                        | StatKey::PercentBoost50To75
                        | StatKey::PercentBoost75To100
                )
            },
            approx_abs(1.0),
        )
        .with_rule(
            "movement timing and percentage fields abs<=1",
            |target| {
                matches!(
                    target.key,
                    StatKey::TimeSupersonicSpeed
                        | StatKey::TimeBoostSpeed
                        | StatKey::TimeSlowSpeed
                        | StatKey::TimeGround
                        | StatKey::TimeLowAir
                        | StatKey::TimeHighAir
                        | StatKey::TimePowerslide
                        | StatKey::PercentSlowSpeed
                        | StatKey::PercentBoostSpeed
                        | StatKey::PercentSupersonicSpeed
                        | StatKey::PercentGround
                        | StatKey::PercentLowAir
                        | StatKey::PercentHighAir
                )
            },
            approx_abs(1.0),
        )
        .with_rule(
            "movement distance/speed fields tolerate external rounding",
            |target| {
                matches!(
                    target.key,
                    StatKey::AvgSpeed
                        | StatKey::AvgSpeedPercentage
                        | StatKey::TotalDistance
                        | StatKey::AvgPowerslideDuration
                )
            },
            |actual, expected, target| {
                let tol = match target.key {
                    StatKey::AvgSpeed => 5.0,
                    StatKey::AvgSpeedPercentage => 0.5,
                    StatKey::TotalDistance => 2500.0,
                    StatKey::AvgPowerslideDuration => 0.1,
                    _ => 0.0,
                };
                (actual - expected).abs() <= tol
            },
        )
        .with_rule(
            "positioning fields abs<=1 or 50 depending on metric",
            |target| target.domain == StatDomain::Positioning,
            |actual, expected, target| {
                let tol = match target.key {
                    StatKey::AvgDistanceToBall
                    | StatKey::AvgDistanceToBallPossession
                    | StatKey::AvgDistanceToBallNoPossession
                    | StatKey::AvgDistanceToMates => 50.0,
                    _ => 1.0,
                };
                (actual - expected).abs() <= tol
            },
        )
}

#[derive(Debug, Default)]
pub(crate) struct StatMatcher {
    pub(super) mismatches: Vec<String>,
}

impl StatMatcher {
    pub(super) fn compare_field(
        &mut self,
        actual: Option<f64>,
        expected: Option<f64>,
        target: ComparisonTarget,
        config: &MatchConfig,
    ) {
        let Some(expected_value) = expected else {
            return;
        };
        let Some(actual_value) = actual else {
            self.mismatches
                .push(format!("{target}: missing actual value"));
            return;
        };

        let outcome = config.evaluate(actual_value, expected_value, &target);
        if !outcome.matches {
            self.mismatches.push(format!(
                "{target}: actual={actual_value} expected={expected_value} predicate={}",
                outcome.description
            ));
        }
    }

    pub(super) fn missing_player(&mut self, scope: &StatScope) {
        self.mismatches
            .push(format!("{scope}: missing actual player"));
    }

    pub(crate) fn into_mismatches(self) -> Vec<String> {
        self.mismatches
    }
}

#[cfg(test)]
mod tests {
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
}
