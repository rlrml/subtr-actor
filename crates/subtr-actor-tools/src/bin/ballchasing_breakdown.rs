use anyhow::Context;
use clap::Parser;
use subtr_actor_tools::ballchasing::{
    compare_replay_against_ballchasing_json_with_breakdown, recommended_match_config,
};

#[path = "ballchasing_breakdown_deltas.rs"]
mod deltas;
#[path = "ballchasing_breakdown_types.rs"]
mod types;

use deltas::collect_numeric_deltas;
use types::{Args, OutputReport};

fn main() -> anyhow::Result<()> {
    let Args {
        replay_path,
        ballchasing_json_path: json_path,
        output_dir,
    } = Args::parse();

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
