mod common;
mod stats_timeline_collector_shots;

use common::parse_replay;
use stats_timeline_collector_shots::{
    assert_half_volley_events_reconstruct_serialized_partial_sums,
    assert_one_timer_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_one_timer_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.one_timer.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain one-timer events");
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_half_volley_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.half_volley.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain half-volley events");
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}
