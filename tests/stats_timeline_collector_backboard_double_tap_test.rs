mod common;
mod stats_timeline_collector_backboard_double_tap;

use common::parse_replay;
use stats_timeline_collector_backboard_double_tap::{
    assert_backboard_events_reconstruct_serialized_partial_sums,
    assert_double_tap_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_backboard_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.backboard.is_empty(),
        "expected backboard fixture to contain backboard events"
    );
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_double_tap_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.double_tap.is_empty(),
        "expected double-tap fixture to contain double-tap events"
    );
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}
