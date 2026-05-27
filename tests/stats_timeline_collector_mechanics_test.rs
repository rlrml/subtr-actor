mod common;
mod stats_timeline_collector_mechanics;

use common::parse_replay;
use stats_timeline_collector_mechanics::{
    assert_quality_mechanic_events_reconstruct_serialized_partial_sums,
    assert_speed_flip_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_mechanic_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut saw_half_flip_event = false;
    let mut saw_wavedash_event = false;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));

        if timeline.events.half_flip.is_empty() && timeline.events.wavedash.is_empty() {
            continue;
        }

        assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
        saw_half_flip_event |= !timeline.events.half_flip.is_empty();
        saw_wavedash_event |= !timeline.events.wavedash.is_empty();

        if saw_half_flip_event && saw_wavedash_event {
            break;
        }
    }

    assert!(
        saw_half_flip_event,
        "expected at least one fixture to contain a half-flip event"
    );
    assert!(
        saw_wavedash_event,
        "expected at least one fixture to contain a wavedash event"
    );
}

#[test]
fn test_speed_flip_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.speed_flip.is_empty(),
        "expected speed-flip fixture to contain speed-flip events"
    );
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}
