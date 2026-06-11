//! Clip-based replacement for `worldcup_ball_kickoff_test.rs`.
//!
//! The original asserted that all five kickoffs of a World Cup-build replay
//! resolve a first touch and outcome (the regression: `Ball_WorldCup` was
//! missing from `BALL_TYPES`, so no touch candidates were generated). This
//! reproduces the assertion for every kickoff on its own clip. The opening
//! kickoff is clipped from frame 0 (no synthetic keyframe); the rest are seeded
//! through the keyframe.

mod common;

use subtr_actor::{
    clip_replay_around_times, EventPayload, KickoffEvent, KickoffOutcome, ReplayClip,
    StatsTimelineCollector,
};

const WORLDCUP_REPLAY: &str = "assets/replay-format-2026-06-02-v868-32-net11-worldcup-ball.replay";

/// Kickoff start times measured from a full-replay run, in replay seconds.
const KICKOFF_START_TIMES: [f32; 5] = [13.94, 37.01, 62.30, 95.98, 141.84];

fn assert_clip_kickoff_resolves(clip: &ReplayClip, expected_start_time: f32) {
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&clip.to_replay())
        .expect("expected a stats timeline for the World Cup kickoff clip");

    let kickoffs: Vec<&KickoffEvent> = timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::Kickoff(kickoff) => Some(kickoff.as_ref()),
            _ => None,
        })
        .collect();
    let kickoff = kickoffs
        .iter()
        .find(|kickoff| (kickoff.start_time - expected_start_time).abs() < 1.0)
        .unwrap_or_else(|| {
            panic!(
                "expected a kickoff starting near {expected_start_time} in the clip; got {:?}",
                kickoffs
                    .iter()
                    .map(|kickoff| kickoff.start_time)
                    .collect::<Vec<_>>()
            )
        });

    assert!(
        kickoff.first_touch_time.is_some(),
        "kickoff starting at {} should resolve a first touch (none were resolved before \
         Ball_WorldCup was recognized as a ball archetype)",
        kickoff.start_time
    );
    assert_ne!(
        kickoff.outcome,
        KickoffOutcome::Unknown,
        "kickoff starting at {} should resolve an outcome",
        kickoff.start_time
    );
    let duration = kickoff.end_time - kickoff.start_time;
    assert!(
        duration < 12.0,
        "kickoff starting at {} should end shortly after the first touch, lasted {duration}s \
         (windows ran to the next goal before the fix)",
        kickoff.start_time
    );
}

#[test]
fn clip_worldcup_all_kickoffs_resolve_first_touches() {
    let replay = common::parse_replay(WORLDCUP_REPLAY);

    for (index, &start_time) in KICKOFF_START_TIMES.iter().enumerate() {
        // The opening kickoff is clipped from the very start of the replay
        // (real_start == 0, no synthetic keyframe); the rest are seeded by the
        // keyframe with warm-up before the countdown.
        let lead_in = if index == 0 { 0 } else { 90 };
        let clip = clip_replay_around_times(
            &replay,
            (start_time - 5.0).max(0.0),
            start_time + 8.0,
            lead_in,
            90,
        )
        .expect("clip should build");
        if index == 0 {
            assert_eq!(
                clip.provenance.synthetic_frame_count, 0,
                "the opening kickoff clip should need no synthetic keyframe"
            );
        } else {
            assert_eq!(
                clip.provenance.synthetic_frame_count, 1,
                "mid-game kickoff clips should be seeded by a synthetic keyframe"
            );
        }
        assert_clip_kickoff_resolves(&clip, start_time);
    }
}
