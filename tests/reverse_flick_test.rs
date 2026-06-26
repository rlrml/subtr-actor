//! Ground-truth regression for reverse-flick classification.
//!
//! `reverse-flick-vs-backflip-2026-06-26.replay` is a controlled replay recorded
//! for this purpose: the player performs a run of *non*-reverse dodges (plain
//! backflips, forward/side flicks, vertical pops) before the first goal, then a
//! run of real reverse flicks after it. Reviewed ground truth (from the recording
//! player): every dodge before the first goal is NOT a reverse flick, every dodge
//! after it IS. This pins the reverse classifier — `dodge_forward_back`,
//! `launch_forward_alignment`, `launch_vertical_fraction`, and `underside_rotation`
//! gates in `FlickCalculator::classify_dodge` — against that split.

mod common;

use subtr_actor::{EventPayload, FlickEvent, GoalContextEvent, StatsTimelineEventCollector};

const REVERSE_FLICK_REPLAY: &str = "assets/reverse-flick-vs-backflip-2026-06-26.replay";

#[test]
fn reverse_flicks_only_occur_after_the_first_goal() {
    let replay = common::parse_replay(REVERSE_FLICK_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build from the reverse-flick replay");

    let goals: Vec<&GoalContextEvent> =
        common::event_payloads(&timeline, |payload| match payload {
            EventPayload::GoalContext(event) => Some(event),
            _ => None,
        });
    let first_goal_time = goals
        .iter()
        .map(|goal| goal.time)
        .fold(f32::INFINITY, f32::min);
    assert!(
        first_goal_time.is_finite(),
        "expected at least one goal in the reverse-flick replay"
    );

    let flicks: Vec<&FlickEvent> = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::Flick(event) => Some(event),
        _ => None,
    });
    assert!(
        flicks
            .iter()
            .any(|flick| flick.dodge_time < first_goal_time),
        "expected non-reverse flicks before the first goal; got {flicks:#?}"
    );
    assert!(
        flicks
            .iter()
            .any(|flick| flick.dodge_time > first_goal_time),
        "expected reverse flicks after the first goal; got {flicks:#?}"
    );

    for flick in &flicks {
        if flick.dodge_time < first_goal_time {
            assert_ne!(
                flick.kind,
                "reverse",
                "flick at {:.2}s is before the first goal ({first_goal_time:.2}s) and must not be \
                 reverse (launch_vertical_fraction={:.3}, underside_rotation={:.3}, \
                 launch_forward_alignment={:.3}); got {flick:#?}",
                flick.dodge_time,
                flick.launch_vertical_fraction,
                flick.underside_rotation,
                flick.launch_forward_alignment,
            );
        } else {
            assert_eq!(
                flick.kind,
                "reverse",
                "flick at {:.2}s is after the first goal ({first_goal_time:.2}s) and must be \
                 reverse (launch_vertical_fraction={:.3}, underside_rotation={:.3}, \
                 launch_forward_alignment={:.3}); got {flick:#?}",
                flick.dodge_time,
                flick.launch_vertical_fraction,
                flick.underside_rotation,
                flick.launch_forward_alignment,
            );
        }
    }
}
