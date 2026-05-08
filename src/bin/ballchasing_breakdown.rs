use std::path::PathBuf;

use anyhow::Context;
use serde::Serialize;
use serde_json::Value;
use subtr_actor::ballchasing::{
    compare_replay_against_ballchasing_json_with_breakdown, recommended_match_config,
};

#[derive(Debug, Serialize)]
struct NumericDelta {
    path: String,
    actual: f64,
    expected: f64,
    delta: f64,
    abs_delta: f64,
}

#[derive(Debug, Serialize)]
struct OutputReport {
    is_match: bool,
    mismatch_count: usize,
    mismatches: Vec<String>,
    deltas: Vec<NumericDelta>,
}

fn collect_numeric_deltas(
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

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let replay_path = args.next().map(PathBuf::from).context(
        "Usage: ballchasing_breakdown <replay-path> <ballchasing-json-path> [output-dir]",
    )?;
    let json_path = args.next().map(PathBuf::from).context(
        "Usage: ballchasing_breakdown <replay-path> <ballchasing-json-path> [output-dir]",
    )?;
    let output_dir = args.next().map(PathBuf::from);

    let breakdown = compare_replay_against_ballchasing_json_with_breakdown(
        &replay_path,
        &json_path,
        &recommended_match_config(),
    )?;

    let mut deltas = Vec::new();
    collect_numeric_deltas(
        "",
        &breakdown.comparable_stats.actual,
        &breakdown.comparable_stats.expected,
        &mut deltas,
    );
    deltas.sort_by(|left, right| {
        right
            .abs_delta
            .partial_cmp(&left.abs_delta)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let report = OutputReport {
        is_match: breakdown.is_match,
        mismatch_count: breakdown.mismatches.len(),
        mismatches: breakdown.mismatches,
        deltas,
    };

    if let Some(output_dir) = output_dir {
        std::fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create {}", output_dir.display()))?;
        std::fs::write(
            output_dir.join("actual.comparable.json"),
            serde_json::to_vec_pretty(&breakdown.comparable_stats.actual)?,
        )?;
        std::fs::write(
            output_dir.join("expected.comparable.json"),
            serde_json::to_vec_pretty(&breakdown.comparable_stats.expected)?,
        )?;
        std::fs::write(
            output_dir.join("comparison-breakdown.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!(
            "wrote {} mismatches and {} numeric deltas to {}",
            report.mismatch_count,
            report.deltas.len(),
            output_dir.display()
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}
