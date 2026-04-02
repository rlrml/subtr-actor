use std::path::Path;

use anyhow::Context;
use serde_json::Value;

use super::comparison::{
    build_actual_comparable_stats, build_expected_comparable_stats, compute_comparable_stats,
    MatchConfig, StatMatcher,
};
use super::report::BallchasingComparisonReport;
use crate::*;

pub fn parse_replay_bytes(data: &[u8]) -> anyhow::Result<boxcars::Replay> {
    boxcars::ParserBuilder::new(data)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .context("Failed to parse replay")
}

pub fn parse_replay_file(path: impl AsRef<Path>) -> anyhow::Result<boxcars::Replay> {
    let path = path.as_ref();
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read replay file: {}", path.display()))?;
    parse_replay_bytes(&data).with_context(|| format!("Failed to parse replay: {}", path.display()))
}

pub fn compare_replay_against_ballchasing(
    replay: &boxcars::Replay,
    ballchasing: &Value,
    config: &MatchConfig,
) -> SubtrActorResult<BallchasingComparisonReport> {
    let computed = compute_comparable_stats(replay)?;
    let actual = build_actual_comparable_stats(&computed);
    let expected = build_expected_comparable_stats(ballchasing);

    let mut matcher = StatMatcher::default();
    expected.compare(&actual, &mut matcher, config);
    Ok(BallchasingComparisonReport {
        mismatches: matcher.into_mismatches(),
    })
}

pub fn compare_replay_against_ballchasing_json(
    replay_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
    config: &MatchConfig,
) -> anyhow::Result<BallchasingComparisonReport> {
    let replay_path = replay_path.as_ref();
    let json_path = json_path.as_ref();
    let replay = parse_replay_file(replay_path)?;
    let json_file = std::fs::File::open(json_path)
        .with_context(|| format!("Failed to open ballchasing json: {}", json_path.display()))?;
    let ballchasing: Value = serde_json::from_reader(json_file)
        .with_context(|| format!("Failed to parse ballchasing json: {}", json_path.display()))?;

    compare_replay_against_ballchasing(&replay, &ballchasing, config)
        .map_err(|error| anyhow::Error::new(error.variant))
}

pub fn compare_fixture_directory(
    path: &Path,
    config: &MatchConfig,
) -> anyhow::Result<BallchasingComparisonReport> {
    let replay_path = path.join("replay.replay");
    let json_path = path.join("ballchasing.json");
    compare_replay_against_ballchasing_json(&replay_path, &json_path, config)
}
