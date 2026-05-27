use super::super::model::ComparisonTarget;

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

pub(in crate::ballchasing::comparison::config) struct MatchOutcome<'a> {
    pub(in crate::ballchasing::comparison::config) matches: bool,
    pub(in crate::ballchasing::comparison::config) description: &'a str,
}

impl MatchConfig {
    pub(in crate::ballchasing::comparison::config) fn exact() -> Self {
        Self::default()
    }

    pub(in crate::ballchasing::comparison::config) fn with_rule<S, P>(
        mut self,
        description: impl Into<String>,
        selector: S,
        predicate: P,
    ) -> Self
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

    pub(in crate::ballchasing::comparison::config) fn evaluate<'a>(
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

pub(in crate::ballchasing::comparison) fn approx_abs(
    abs_tol: f64,
) -> impl Fn(f64, f64, &ComparisonTarget) -> bool {
    move |actual, expected, _| (actual - expected).abs() <= abs_tol
}
