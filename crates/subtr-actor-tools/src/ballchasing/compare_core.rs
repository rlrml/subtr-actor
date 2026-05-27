use serde_json::Value;

use super::super::comparison::{
    build_actual_comparable_stats, build_expected_comparable_stats, compute_comparable_stats,
    MatchConfig, StatMatcher,
};
use super::super::report::BallchasingComparisonReport;
use super::types::{BallchasingComparableStats, BallchasingComparisonBreakdown};
use subtr_actor::*;

pub fn compare_replay_against_ballchasing(
    replay: &boxcars::Replay,
    ballchasing: &Value,
    config: &MatchConfig,
) -> SubtrActorResult<BallchasingComparisonReport> {
    let (mismatches, _, _) = compare_replay_stats(replay, ballchasing, config)?;
    Ok(BallchasingComparisonReport { mismatches })
}

pub fn compare_replay_against_ballchasing_with_breakdown(
    replay: &boxcars::Replay,
    ballchasing: &Value,
    config: &MatchConfig,
) -> SubtrActorResult<BallchasingComparisonBreakdown> {
    let (mismatches, actual, expected) = compare_replay_stats(replay, ballchasing, config)?;
    Ok(BallchasingComparisonBreakdown {
        is_match: mismatches.is_empty(),
        mismatches,
        comparable_stats: BallchasingComparableStats {
            actual: serde_json::to_value(&actual).expect("comparable stats should serialize"),
            expected: serde_json::to_value(&expected).expect("comparable stats should serialize"),
        },
    })
}

fn compare_replay_stats(
    replay: &boxcars::Replay,
    ballchasing: &Value,
    config: &MatchConfig,
) -> SubtrActorResult<(
    Vec<String>,
    super::super::comparison::ComparableReplayStats,
    super::super::comparison::ComparableReplayStats,
)> {
    let computed = compute_comparable_stats(replay)?;
    let actual = build_actual_comparable_stats(&computed);
    let expected = build_expected_comparable_stats(ballchasing);
    let mut matcher = StatMatcher::default();
    expected.compare(&actual, &mut matcher, config);
    Ok((matcher.into_mismatches(), actual, expected))
}
