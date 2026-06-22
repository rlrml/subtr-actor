//! Clip-based recall regression for the flick detector.
//!
//! Pins CaleMaCar's dribble in the reviewed rocket-sense replay
//! `019eeca3-fb27-7c60-9393-6ca9a0bf9902`: three flicks in quick succession,
//! each ending a carry with a dodge-powered launch. The middle flick's launch
//! touch is sampled ~0.23s *before* the dodge component's active byte
//! replicates — well past the old 0.12s lead tolerance and the 0.15s impulse
//! window — so it used to be dropped while its two neighbours (whose dodge byte
//! led the touch) were detected. The lag-tolerant detection must now recover all
//! three. Following the workflow in `clip_flip_reset_test`: find the case on the
//! full replay once, then pin it on a small clip so the test only processes the
//! frames that matter.

mod common;

use subtr_actor::{
    EventPayload, FlickEvent, ReplayStatsTimelineScaffold, StatsTimelineEventCollector,
    clip_replay_around,
};

const FLICK_SEQUENCE_REPLAY: &str = "assets/calemacar-dribble-flick-sequence-2026-06-22.replay";

// Source-replay frames spanning the three-flick dribble. The first flick's carry
// setup starts near frame 4495 and the third flick's dodge lands at frame 4748;
// the region brackets all three with a little margin.
const SEQUENCE_START_FRAME: usize = 4470;
const SEQUENCE_END_FRAME: usize = 4760;

// Absolute dodge times of the three flicks (seconds). The middle one is the
// regression target: its launch touch precedes the dodge byte by ~0.23s.
const FIRST_FLICK_DODGE_TIME: f32 = 225.964;
const MISSED_FLICK_DODGE_TIME: f32 = 230.517;
const THIRD_FLICK_DODGE_TIME: f32 = 235.901;

fn clip_timeline(region_start: usize, region_end: usize) -> ReplayStatsTimelineScaffold {
    let replay = common::parse_replay(FLICK_SEQUENCE_REPLAY);
    let clip =
        clip_replay_around(&replay, region_start, region_end, 150, 150).expect("clip builds");
    StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("stats timeline should build from a flick-sequence clip")
}

fn flicks_near(flicks: &[&FlickEvent], dodge_time: f32) -> usize {
    flicks
        .iter()
        .filter(|flick| (flick.dodge_time - dodge_time).abs() < 0.15)
        .count()
}

#[test]
fn clip_detects_all_three_flicks_in_calemacar_dribble() {
    let timeline = clip_timeline(SEQUENCE_START_FRAME, SEQUENCE_END_FRAME);
    let flicks: Vec<&FlickEvent> =
        common::event_payloads_by_stream(&timeline, "flick", |payload| match payload {
            EventPayload::Flick(event) => Some(event),
            _ => None,
        });

    // The middle flick — previously dropped because its launch touch led the
    // dodge byte by more than the lag tolerance — is the regression target.
    assert_eq!(
        flicks_near(&flicks, MISSED_FLICK_DODGE_TIME),
        1,
        "expected exactly one flick for the previously-missed middle dodge \
         (~{MISSED_FLICK_DODGE_TIME}s); got {flicks:#?}"
    );

    // Its two neighbours must still be detected, and each dodge must yield
    // exactly one flick (no double-counting from the extended pending window).
    assert_eq!(
        flicks_near(&flicks, FIRST_FLICK_DODGE_TIME),
        1,
        "expected exactly one flick for the first dodge (~{FIRST_FLICK_DODGE_TIME}s); \
         got {flicks:#?}"
    );
    assert_eq!(
        flicks_near(&flicks, THIRD_FLICK_DODGE_TIME),
        1,
        "expected exactly one flick for the third dodge (~{THIRD_FLICK_DODGE_TIME}s); \
         got {flicks:#?}"
    );
}
