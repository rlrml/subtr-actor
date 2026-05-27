use super::super::model::{ComparisonTarget, StatScope};
use super::match_config::MatchConfig;

#[derive(Debug, Default)]
pub(crate) struct StatMatcher {
    pub(super) mismatches: Vec<String>,
}

impl StatMatcher {
    pub(in crate::ballchasing::comparison) fn compare_field(
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

    pub(in crate::ballchasing::comparison) fn missing_player(&mut self, scope: &StatScope) {
        self.mismatches
            .push(format!("{scope}: missing actual player"));
    }

    pub(crate) fn into_mismatches(self) -> Vec<String> {
        self.mismatches
    }
}
