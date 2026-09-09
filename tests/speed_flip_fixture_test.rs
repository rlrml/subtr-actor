//! Whole-replay precision/recall fixtures for speed-flip detection.
//!
//! Unlike `clip_speed_flip_test` (which pins individual reviewed cases on small
//! clips), these process a full replay that was *recorded for ground truth* and
//! assert on the whole-replay speed-flip count. They are the precision (no false
//! positives) and recall (no false negatives) targets for the detector.

use subtr_actor::{EventPayload, SpeedFlipEvent, StatsTimelineEventCollector};

/// A replay recorded deliberately with **zero** speed flips — only confounding
/// dodges (diagonal flips, side flips, forward dodges) meant to bait the
/// detector. Any speed flip found here is a false positive.
const NO_SPEED_FLIPS_REPLAY: &str = "assets/speed-flip-confounders-none-2026-06-20.replay";

/// A replay recorded as a speed-flip drill: ~every dodge is a speed flip (a few
/// were flubbed). The recorder counted ~40 dodges, almost all speed flips, so a
/// healthy detector should recover the large majority. This is the recall target.
const ALL_SPEED_FLIPS_REPLAY: &str = "assets/speed-flip-recall-drill-2026-06-20.replay";
const ALL_SPEED_FLIPS_MIN_RECALL: f64 = 0.85;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn speed_flip_events(replay: &boxcars::Replay) -> Vec<SpeedFlipEvent> {
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(replay)
        .expect("stats timeline should build");
    timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::SpeedFlip(event) => Some(event.clone()),
            _ => None,
        })
        .collect()
}

fn dodge_and_speed_flip_counts(replay: &boxcars::Replay) -> (usize, usize) {
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(replay)
        .expect("stats timeline should build");
    let mut dodges = 0;
    let mut speed_flips = 0;
    for event in &timeline.events.events {
        match &event.payload {
            EventPayload::Dodge(_) => dodges += 1,
            EventPayload::SpeedFlip(_) => speed_flips += 1,
            _ => {}
        }
    }
    (dodges, speed_flips)
}

#[test]
#[ignore = "PRECISION target (run with `--ignored`): the confounder replay should yield \
            0 speed flips. Slow (full replay)."]
fn confounder_replay_has_no_speed_flips() {
    let replay = parse_replay(NO_SPEED_FLIPS_REPLAY);
    let speed_flips = speed_flip_events(&replay);
    assert!(
        speed_flips.is_empty(),
        "expected 0 speed flips in the confounder replay, found {}: {:#?}",
        speed_flips.len(),
        speed_flips
            .iter()
            .map(|event| {
                (
                    event.time,
                    event.confidence,
                    event.min_travel_alignment,
                    event.max_forward_deviation_degrees,
                    event.roll_sweep_degrees,
                )
            })
            .collect::<Vec<_>>(),
    );
}

#[test]
#[ignore = "RECALL target (run with `--ignored`): nearly every dodge in this drill is a \
            speed flip. Slow (full replay)."]
fn speed_flip_drill_recovers_most_dodges() {
    let replay = parse_replay(ALL_SPEED_FLIPS_REPLAY);
    let (dodges, speed_flips) = dodge_and_speed_flip_counts(&replay);
    let recall = speed_flips as f64 / dodges as f64;
    assert!(
        recall >= ALL_SPEED_FLIPS_MIN_RECALL,
        "expected >= {:.0}% of the {} dodges to be detected as speed flips, got {} ({:.0}%)",
        ALL_SPEED_FLIPS_MIN_RECALL * 100.0,
        dodges,
        speed_flips,
        recall * 100.0,
    );
}
