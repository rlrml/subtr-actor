//! Clip-based variant of `worldcup_ball_kickoff_test.rs`.
//!
//! Covers two of the replay's five kickoffs (verified against a full-replay
//! run): the opening kickoff, clipped from frame 0 (no synthetic keyframe), and
//! a mid-game kickoff seeded through the synthetic keyframe — both on the
//! newer net11 World Cup replay format. The full-replay original is
//! `#[ignore]`d as slow; the clips run by default.
//!
//! See the original test for the regression background (`Ball_WorldCup` missing
//! from `BALL_TYPES` left every kickoff without a first touch).

mod common;

use subtr_actor::{
    clip_replay_around_times, EventPayload, KickoffEvent, KickoffOutcome, ReplayClip,
    StatsTimelineCollector,
};

const WORLDCUP_REPLAY: &str = "assets/replay-format-2026-06-02-v868-32-net11-worldcup-ball.replay";

/// Kickoff start times measured from a full-replay run.
const OPENING_KICKOFF_START_TIME: f32 = 13.94;
const MID_GAME_KICKOFF_START_TIME: f32 = 62.30;

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
fn clip_worldcup_opening_kickoff_resolves_first_touch() {
    let replay = common::parse_replay(WORLDCUP_REPLAY);
    // Clip from the very start of the replay: real_start == 0, so the clip has
    // no synthetic keyframe and the opening countdown plays out unmodified.
    let clip = clip_replay_around_times(&replay, 0.0, OPENING_KICKOFF_START_TIME + 8.0, 0, 90)
        .expect("clip should build");
    assert_eq!(
        clip.provenance.synthetic_frame_count, 0,
        "a clip starting at frame 0 needs no synthetic keyframe"
    );
    assert_clip_kickoff_resolves(&clip, OPENING_KICKOFF_START_TIME);
}

#[test]
fn clip_worldcup_mid_game_kickoff_resolves_first_touch() {
    let replay = common::parse_replay(WORLDCUP_REPLAY);
    let clip = clip_replay_around_times(
        &replay,
        MID_GAME_KICKOFF_START_TIME - 5.0,
        MID_GAME_KICKOFF_START_TIME + 8.0,
        90,
        90,
    )
    .expect("clip should build");
    assert_eq!(clip.provenance.synthetic_frame_count, 1);
    assert_clip_kickoff_resolves(&clip, MID_GAME_KICKOFF_START_TIME);
}
