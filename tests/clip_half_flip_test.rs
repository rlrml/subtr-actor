//! Clip-backed half-flip recall regression.
//!
//! This replay has a late freeplay-style sequence with repeated half flips
//! that used to be mostly missed. The clip keeps the test fast while still
//! exercising the real replay pipeline and event collector.

mod common;

use subtr_actor::{
    EventPayload, HalfFlipEvent, ReplayStatsTimelineScaffold, StatsTimelineEventCollector,
    clip_replay_around_times,
};

const HALF_FLIP_RECALL_REPLAY: &str = "assets/half-flip-recall-drill-2026-06-20.replay";

fn half_flip_events(timeline: &ReplayStatsTimelineScaffold) -> Vec<&HalfFlipEvent> {
    timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::HalfFlip(event) => Some(event),
            _ => None,
        })
        .collect()
}

#[test]
fn clip_late_half_flip_drill_detects_substantial_recall() {
    let replay = common::parse_replay(HALF_FLIP_RECALL_REPLAY);

    // The attached replay ends with a repeated half-flip drill. This window
    // covers the dense cluster around 265s-283s without scanning the whole
    // replay.
    let clip = clip_replay_around_times(&replay, 265.0, 283.5, 120, 90)
        .expect("half-flip drill clip should build");
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("stats timeline should build from the half-flip drill clip");

    let events = half_flip_events(&timeline);
    let event_summary = events
        .iter()
        .map(|event| {
            format!(
                "t={:.3}s frame={} speed={:.1} z={:.1} rev={:.3} vert={:.3} conf={:.3}",
                event.time,
                event.frame,
                event.start_speed,
                event.start_position[2],
                event.best_forward_reversal,
                event.max_forward_vertical,
                event.confidence
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    assert!(
        events.len() >= 7,
        "expected substantial half-flip recall in the late drill clip; \
         found {} events: [{}]",
        events.len(),
        event_summary
    );
    assert!(
        events.iter().any(|event| event.start_speed < 600.0),
        "the regression clip should include slow/reorienting half flips; \
         found: [{event_summary}]"
    );
    assert!(
        events.iter().any(|event| event.start_position[2] > 70.0),
        "the regression clip should include low-air half flips, not only fully grounded starts; \
         found: [{event_summary}]"
    );
}
