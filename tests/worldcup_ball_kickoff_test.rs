//! Regression test for ball detection on World Cup update replays.
//!
//! The June 2026 World Cup update introduced a new ball archetype,
//! `Archetypes.Ball.Ball_WorldCup`, which was absent from `BALL_TYPES`. With no
//! ball actor resolved, no touch candidates were ever generated, so every
//! kickoff ended with `first_touch_time = None`, `outcome = Unknown`, and a
//! window that only closed on the next goal-replay state — inflating taker
//! boost usage over tens of seconds of non-kickoff play.
//!
//! `assets/replay-format-2026-06-02-v868-32-net11-worldcup-ball.replay` is a
//! ranked doubles match played on the World Cup build (EuroStadium_P,
//! BuildVersion 260602.75104.519749) where all five kickoffs exhibited the
//! failure.

mod common;

use subtr_actor::{EventPayload, KickoffEvent, KickoffOutcome, StatsTimelineCollector};

const WORLDCUP_REPLAY: &str = "assets/replay-format-2026-06-02-v868-32-net11-worldcup-ball.replay";

#[test]
#[ignore = "replay-backed ball detection parity is slow; run explicitly when changing ball or touch resolution"]
fn worldcup_ball_kickoffs_resolve_first_touches() {
    let replay = common::parse_replay(WORLDCUP_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("expected a stats timeline for the World Cup replay");

    let kickoffs: Vec<&KickoffEvent> = timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::Kickoff(kickoff) => Some(kickoff.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(
        kickoffs.len(),
        5,
        "expected the five kickoffs of the World Cup replay"
    );

    for kickoff in kickoffs {
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
}
