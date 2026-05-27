mod common;
mod stats_timeline_collector_bump_demo;

use common::parse_replay;
use stats_timeline_collector_bump_demo::{
    assert_bump_events_reconstruct_serialized_partial_sums,
    assert_demo_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_bump_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/post-eac-ranked-standard-2026-04-28.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.bump.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain bump events");
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_demo_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline.events.timeline.iter().any(|event| matches!(
            event.kind,
            TimelineEventKind::Kill | TimelineEventKind::Death
        )),
        "expected demo fixture to contain kill/death timeline events"
    );
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}
