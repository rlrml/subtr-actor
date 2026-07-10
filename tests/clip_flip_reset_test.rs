//! Clip-based regression for flip-reset goal tagging.
//!
//! Pins CaleMaCar's 9th goal in the reviewed rocket-sense duel
//! `019ee9cb-40a1-7c30-9a2e-a846172dd869`: an on-ball flip reset converted by a
//! flip-into-ball finish where the conversion touch is sampled a frame before
//! the dodge component's active byte replicates. The reset must still be
//! confirmed (so the goal earns `FlipResetGoal`); a strict "dodge active exactly
//! at the touch" gate misses it. Following the workflow in
//! `clip_speed_flip_test`: find the case on the full replay once, then pin it on
//! a small clip so the test only processes the frames that matter.

mod common;

use subtr_actor::{
    EventPayload, FlipResetEvent, GoalTagEventStream, GoalTagKind, ReplayStatsTimelineScaffold,
    StatsTimelineEventCollector, clip_replay_around,
};

const FLIP_RESET_GOAL_REPLAY: &str = "assets/calemacar-flip-reset-goal-9-2026-06-21.replay";
const SQUISHY_FLIP_RESET_MISS_REPLAY: &str =
    "assets/squishy-ranked-doubles-flip-reset-miss-2026-03-01.replay";
// A short rocket-sense duel clip (`019f49be-2b0e-79e3-8745-4ea9af20d6cd`) with
// two double-reset aerials by Panzinii; the second one scores. In each aerial
// the first reset is converted by a dodge whose drag carries the car back into
// the ball, and that same contact grants the second reset (dropping the dodge
// byte on the contact frame) — the case that used to resolve as Superseded.
const DOUBLE_FLIP_RESET_REPLAY: &str = "assets/panzinii-double-flip-reset-2026-07-09.replay";
// Reset times (absolute seconds) of the two on-ball resets in each aerial.
const FIRST_AERIAL_RESET_TIMES: [f32; 2] = [42.765, 44.079];
const SECOND_AERIAL_RESET_TIMES: [f32; 2] = [68.321, 69.641];

// Source-replay frames: the on-ball reset lands at 6520 and the converting
// flip-into-ball touch at 6521; the goal is scored at frame 6555. The tail must
// reach past the goal so goal-context tagging closes out.
const RESET_SOURCE_FRAME: usize = 6520;
const GOAL_SOURCE_FRAME: usize = 6555;
const RESET_TIME: f32 = 316.722;
const SCORING_TOUCH_TIME: f32 = 316.768;

fn clip_timeline(region_start: usize, region_end: usize) -> ReplayStatsTimelineScaffold {
    let replay = common::parse_replay(FLIP_RESET_GOAL_REPLAY);
    let clip =
        clip_replay_around(&replay, region_start, region_end, 150, 200).expect("clip builds");
    StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("stats timeline should build from a flip-reset clip")
}

#[test]
fn clip_tags_calemacar_ninth_goal_as_flip_reset() {
    let timeline = clip_timeline(RESET_SOURCE_FRAME, GOAL_SOURCE_FRAME);

    // The lag-tolerant confirmation should emit a confirmed flip reset at the
    // conversion touch, sourced from the on-ball reset one frame earlier.
    let confirmed: Vec<&FlipResetEvent> =
        common::event_payloads_by_stream(&timeline, "flip_reset", |payload| match payload {
            EventPayload::FlipReset(event) => Some(event),
            _ => None,
        });
    assert!(
        confirmed
            .iter()
            .any(|event| (event.time - SCORING_TOUCH_TIME).abs() < 0.1
                && (event.reset_time - RESET_TIME).abs() < 0.1),
        "expected a confirmed flip reset at the conversion touch; got {confirmed:#?}"
    );

    // The goal inside the clip must carry the FlipResetGoal tag.
    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });
    assert!(
        !goal_context.is_empty(),
        "expected the flip-reset goal inside the clip; got no goal context events"
    );
    assert!(
        goal_context.iter().any(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::FlipResetGoal)),
        "expected the goal inside the clip to be tagged as a flip reset; got {goal_context:#?}"
    );
}

#[test]
fn double_reset_aerials_credit_every_reset() {
    let replay = common::parse_replay(DOUBLE_FLIP_RESET_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build for the double flip reset replay");

    let confirmed: Vec<&FlipResetEvent> =
        common::event_payloads_by_stream(&timeline, "flip_reset", |payload| match payload {
            EventPayload::FlipReset(event) => Some(event),
            _ => None,
        });
    for reset_time in FIRST_AERIAL_RESET_TIMES
        .iter()
        .chain(SECOND_AERIAL_RESET_TIMES.iter())
    {
        assert!(
            confirmed
                .iter()
                .any(|event| (event.reset_time - reset_time).abs() < 0.1),
            "expected a confirmed flip reset from the reset at {reset_time}; got {confirmed:#?}"
        );
    }
    assert_eq!(
        confirmed.len(),
        4,
        "expected exactly two confirmed flip resets per double-reset aerial; got {confirmed:#?}"
    );
}

#[test]
fn double_reset_goal_counts_both_resets() {
    let replay = common::parse_replay(DOUBLE_FLIP_RESET_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build for the double flip reset replay");

    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });
    let flip_reset_tag = goal_context
        .iter()
        .flat_map(|goal| goal.tags.iter())
        .find(|tag| tag.kind() == GoalTagKind::FlipResetGoal)
        .expect("expected the double-reset goal to carry a FlipResetGoal tag");

    let metadata = flip_reset_tag.metadata();
    let count_detail = metadata
        .details
        .iter()
        .find(|detail| detail.key == "flip_reset_count")
        .expect("expected the FlipResetGoal tag to carry a flip_reset_count detail");
    assert_eq!(
        count_detail.value, "2",
        "a double-reset goal should count both flip resets; got {metadata:#?}"
    );

    let flip_reset_refs = metadata
        .related_events
        .iter()
        .filter(|event_ref| event_ref.stream == GoalTagEventStream::FlipReset)
        .count();
    assert_eq!(
        flip_reset_refs, 2,
        "expected both confirmed flip resets as related events; got {metadata:#?}"
    );
}

#[test]
fn squishy_last_goal_is_tagged_as_flip_reset() {
    let replay = common::parse_replay(SQUISHY_FLIP_RESET_MISS_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build for Squishy flip-reset replay");

    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });
    let last_goal = goal_context
        .last()
        .expect("expected at least one goal context event");

    assert!(
        last_goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::FlipResetGoal),
        "expected the last goal to be tagged as a flip reset; got {last_goal:#?}"
    );
}
