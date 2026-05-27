use std::path::Path;

use anyhow::Context;
use serde_json::Value;

use super::super::comparison::MatchConfig;
use super::super::report::BallchasingComparisonReport;
use super::core::{
    compare_replay_against_ballchasing, compare_replay_against_ballchasing_with_breakdown,
};
use super::parse::parse_replay_file;
use super::types::BallchasingComparisonBreakdown;

pub fn compare_replay_against_ballchasing_json_with_breakdown(
    replay_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
    config: &MatchConfig,
) -> anyhow::Result<BallchasingComparisonBreakdown> {
    let replay = parse_replay_file(replay_path)?;
    let ballchasing = read_ballchasing_json(json_path)?;

    compare_replay_against_ballchasing_with_breakdown(&replay, &ballchasing, config)
        .map_err(|error| anyhow::Error::new(error.variant))
}

pub fn compare_replay_against_ballchasing_json(
    replay_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
    config: &MatchConfig,
) -> anyhow::Result<BallchasingComparisonReport> {
    let replay = parse_replay_file(replay_path)?;
    let ballchasing = read_ballchasing_json(json_path)?;

    compare_replay_against_ballchasing(&replay, &ballchasing, config)
        .map_err(|error| anyhow::Error::new(error.variant))
}

pub fn compare_fixture_directory(
    path: &Path,
    config: &MatchConfig,
) -> anyhow::Result<BallchasingComparisonReport> {
    let (replay_path, json_path) = if path.is_dir() {
        (path.join("replay.replay"), path.join("ballchasing.json"))
    } else {
        (
            path.with_extension("replay"),
            path.with_extension("ballchasing.json"),
        )
    };
    compare_replay_against_ballchasing_json(&replay_path, &json_path, config)
}

fn read_ballchasing_json(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let json_file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open ballchasing json: {}", path.display()))?;
    serde_json::from_reader(json_file)
        .with_context(|| format!("Failed to parse ballchasing json: {}", path.display()))
}
