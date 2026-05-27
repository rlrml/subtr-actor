mod common;
mod stats_timeline_collector_field_state;

use common::parse_replay;
use stats_timeline_collector_field_state::{
    assert_movement_events_reconstruct_serialized_partial_sums,
    assert_positioning_events_reconstruct_serialized_partial_sums,
    assert_possession_events_reconstruct_serialized_partial_sums,
    assert_pressure_events_reconstruct_serialized_partial_sums,
    assert_rotation_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_possession_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.possession.is_empty(),
        "expected possession fixture to contain possession events"
    );
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_pressure_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.pressure.is_empty(),
        "expected pressure fixture to contain pressure events"
    );
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_movement_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.movement.is_empty(),
        "expected movement fixture to contain movement events"
    );
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_positioning_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.positioning.is_empty(),
        "expected positioning fixture to contain positioning events"
    );
    assert_positioning_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_rotation_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.rotation_player.is_empty(),
        "expected rotation fixture to contain rotation player events"
    );
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}
