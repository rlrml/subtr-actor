mod common;

use subtr_actor::{GoalTagKind, StatsCollector, StatsFrameResolution, StatsTimelineCollector};

const THIRD_GOAL_DOUBLE_TAP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
const NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY: &str =
    "assets/nuttrback-double-tap-goal-7-2026-06-01.replay";

#[test]
fn tags_colonelpanic8_third_goal_as_double_tap() {
    let replay = common::parse_replay(THIRD_GOAL_DOUBLE_TAP_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("failed to collect stats timeline for double tap replay");

    assert!(
        timeline.events.double_tap.iter().any(|event| {
            (event.backboard_time - 174.0366).abs() < 0.05 && (event.time - 174.481).abs() < 0.05
        }),
        "expected raw double tap event before the third goal"
    );
    assert!(
        timeline.events.mechanics.iter().any(|event| {
            event.kind == "double_tap"
                && match event.timing {
                    subtr_actor::MechanicTiming::Span {
                        start_time,
                        end_time,
                        ..
                    } => (start_time - 174.0366).abs() < 0.05 && (end_time - 174.481).abs() < 0.05,
                    subtr_actor::MechanicTiming::Moment { .. } => false,
                }
        }),
        "expected generic mechanics stream to include the double tap"
    );
    assert!(
        timeline
            .events
            .goal_tags
            .iter()
            .any(|event| event.goal_index == 2 && event.kind == GoalTagKind::DoubleTapGoal),
        "expected third goal to be tagged as a double tap; got {:?}",
        timeline.events.goal_tags
    );
}

#[test]
fn tags_nuttrback_seventh_goal_as_double_tap() {
    let replay = common::parse_replay(NUTTRBACK_GOAL_7_DOUBLE_TAP_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("failed to collect stats timeline for double tap replay");

    assert!(
        timeline.events.double_tap.iter().any(|event| {
            (event.backboard_time - 343.42084).abs() < 0.05 && (event.time - 343.69733).abs() < 0.05
        }),
        "expected raw double tap event before the seventh goal; got {:?}",
        timeline.events.double_tap
    );
    assert!(
        timeline.events.mechanics.iter().any(|event| {
            event.kind == "double_tap"
                && match event.timing {
                    subtr_actor::MechanicTiming::Span {
                        start_time,
                        end_time,
                        ..
                    } => {
                        (start_time - 343.42084).abs() < 0.05 && (end_time - 343.69733).abs() < 0.05
                    }
                    subtr_actor::MechanicTiming::Moment { .. } => false,
                }
        }),
        "expected generic mechanics stream to include nuttrback's double tap"
    );
    assert!(
        timeline
            .events
            .goal_tags
            .iter()
            .any(|event| event.goal_index == 6 && event.kind == GoalTagKind::DoubleTapGoal),
        "expected seventh goal to be tagged as a double tap; got {:?}",
        timeline.events.goal_tags
    );
}

#[test]
fn dynamic_stats_timeline_value_includes_normalized_mechanics_stream() {
    let replay = common::parse_replay(THIRD_GOAL_DOUBLE_TAP_REPLAY);
    let value = StatsCollector::new()
        .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 1.0 })
        .capture_frames()
        .get_captured_data(&replay)
        .expect("failed to capture stats frames for double tap replay")
        .into_legacy_stats_timeline_value()
        .expect("failed to convert captured stats frames to timeline value");

    let mechanics = value["events"]["mechanics"]
        .as_array()
        .expect("timeline value should expose mechanics as an array");
    assert!(
        mechanics
            .iter()
            .any(|event| event["kind"].as_str() == Some("double_tap")),
        "expected value timeline mechanics stream to include the double tap"
    );
}
