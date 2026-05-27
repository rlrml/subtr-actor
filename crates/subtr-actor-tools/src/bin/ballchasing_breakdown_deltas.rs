use serde_json::Value;

use super::types::NumericDelta;

pub(crate) fn collect_numeric_deltas(
    path: &str,
    actual: &Value,
    expected: &Value,
    deltas: &mut Vec<NumericDelta>,
) {
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            let Some(actual) = actual.as_f64() else {
                return;
            };
            let Some(expected) = expected.as_f64() else {
                return;
            };
            if actual != expected {
                let delta = actual - expected;
                deltas.push(NumericDelta {
                    path: path.to_string(),
                    actual,
                    expected,
                    delta,
                    abs_delta: delta.abs(),
                });
            }
        }
        (Value::Object(actual), Value::Object(expected)) => {
            for (key, expected_value) in expected {
                let child_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                collect_numeric_deltas(
                    &child_path,
                    actual.get(key).unwrap_or(&Value::Null),
                    expected_value,
                    deltas,
                );
            }
        }
        _ => {}
    }
}
