mod common;

use subtr_actor::{EventPayload, EventTiming, GoalTagKind, StatsTimelineEventCollector};

const THIRD_GOAL_DOUBLE_TAP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
const NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY: &str =
    "assets/nuttrback-double-tap-goal-7-2026-06-01.replay";

#[test]
fn tags_colonelpanic8_third_goal_as_double_tap() {
    let replay = common::parse_replay(THIRD_GOAL_DOUBLE_TAP_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("failed to collect stats timeline for double tap replay");
    let double_tap_events =
        common::event_payloads_by_stream(&timeline, "double_tap", |payload| match payload {
            EventPayload::DoubleTap(event) => Some(event),
            _ => None,
        });
    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });

    assert!(
        double_tap_events.iter().any(|event| {
            (event.backboard_time - 174.0366).abs() < 0.05 && (event.time - 174.481).abs() < 0.05
        }),
        "expected raw double tap event before the third goal"
    );
    assert!(
        timeline.events.events.iter().any(|event| {
            event.meta.stream == "double_tap"
                && match event.meta.timing {
                    EventTiming::Span {
                        start_time,
                        end_time,
                        ..
                    } => (start_time - 174.0366).abs() < 0.05 && (end_time - 174.481).abs() < 0.05,
                    EventTiming::Moment { .. } => false,
                }
        }),
        "expected event stream to include the double tap"
    );
    assert!(
        goal_context.get(2).is_some_and(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::DoubleTapGoal)),
        "expected third goal to be tagged as a double tap; got {goal_context:?}"
    );

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
#[ignore = "second full-replay double-tap variant is slow and duplicates default double-tap coverage"]
fn tags_nuttrback_seventh_goal_as_double_tap() {
    let replay = common::parse_replay(NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("failed to collect stats timeline for double tap replay");
    let double_tap_events =
        common::event_payloads_by_stream(&timeline, "double_tap", |payload| match payload {
            EventPayload::DoubleTap(event) => Some(event),
            _ => None,
        });
    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });

    assert!(
        double_tap_events.iter().any(|event| {
            (event.backboard_time - 343.42084).abs() < 0.05 && (event.time - 343.69733).abs() < 0.05
        }),
        "expected raw double tap event before the seventh goal; got {double_tap_events:?}"
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
                        (start_time - 343.42084).abs() < 0.05 && (end_time - 343.69733).abs() < 0.05
                    }
                    EventTiming::Moment { .. } => false,
                }
        }),
        "expected event stream to include nuttrback's double tap"
    );
    assert!(
        goal_context.get(6).is_some_and(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::DoubleTapGoal)),
        "expected seventh goal to be tagged as a double tap; got {goal_context:?}"
    );
}
