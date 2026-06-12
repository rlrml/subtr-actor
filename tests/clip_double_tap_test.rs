//! Clip-based variant of `double_tap_replay_test.rs`.
//!
//! Each case clips a window around the reviewed double tap and the goal it
//! produces. Goal indices shift inside a clip (earlier goals fall outside the
//! window), so the goal-tag assertions match the goal *inside* the clip instead
//! of the source replay's goal ordinal. Notably this makes the nuttrback case —
//! `#[ignore]`d as too slow in its full-replay form — cheap enough to run by
//! default.

mod common;

use subtr_actor::{
    clip_replay_around_times, EventPayload, EventTiming, GoalTagKind, ReplayStatsTimelineScaffold,
    StatsTimelineEventCollector,
};

const THIRD_GOAL_DOUBLE_TAP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
const NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY: &str =
    "assets/nuttrback-double-tap-goal-7-2026-06-01.replay";

struct DoubleTapCase {
    replay_path: &'static str,
    backboard_time: f32,
    touch_time: f32,
}

fn assert_clip_detects_double_tap_goal(case: &DoubleTapCase) -> ReplayStatsTimelineScaffold {
    let replay = common::parse_replay(case.replay_path);
    // Tail long enough to include the goal scored by the double tap plus the
    // goal celebration state that closes out goal context tagging.
    let clip = clip_replay_around_times(&replay, case.backboard_time, case.touch_time, 90, 180)
        .expect("clip should build");
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("failed to collect stats timeline for double tap clip");

    let double_tap_events =
        common::event_payloads_by_stream(&timeline, "double_tap", |payload| match payload {
            EventPayload::DoubleTap(event) => Some(event),
            _ => None,
        });
    assert!(
        double_tap_events.iter().any(|event| {
            (event.backboard_time - case.backboard_time).abs() < 0.05
                && (event.time - case.touch_time).abs() < 0.05
        }),
        "expected raw double tap event in the clip; got {double_tap_events:?}"
    );

    assert!(
        timeline.events.events.iter().any(|event| {
            event.meta.stream == "double_tap"
                && match event.meta.timing {
                    EventTiming::Span {
                        start_time,
                        end_time,
                        ..
                    } => {
                        (start_time - case.backboard_time).abs() < 0.05
                            && (end_time - case.touch_time).abs() < 0.05
                    }
                    EventTiming::Moment { .. } => false,
                }
        }),
        "expected event stream to include the double tap"
    );

    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });
    assert!(
        !goal_context.is_empty(),
        "expected the double tap goal inside the clip; got no goal context events"
    );
    assert!(
        goal_context.iter().any(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::DoubleTapGoal)),
        "expected the goal inside the clip to be tagged as a double tap; got {goal_context:?}"
    );

    timeline
}

#[test]
fn clip_tags_colonelpanic8_third_goal_as_double_tap() {
    let timeline = assert_clip_detects_double_tap_goal(&DoubleTapCase {
        replay_path: THIRD_GOAL_DOUBLE_TAP_REPLAY,
        backboard_time: 174.0366,
        touch_time: 174.481,
    });

    let value = serde_json::to_value(&timeline).expect("event timeline should serialize");
    let events = value["events"]["events"]
        .as_array()
        .expect("timeline value should expose events as an array");
    assert!(
        events
            .iter()
            .any(|event| event["meta"]["stream"].as_str() == Some("double_tap")),
        "expected value timeline event stream to include the double tap"
    );
}

#[test]
fn clip_tags_nuttrback_seventh_goal_as_double_tap() {
    assert_clip_detects_double_tap_goal(&DoubleTapCase {
        replay_path: NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY,
        backboard_time: 343.42084,
        touch_time: 343.69733,
    });
}
