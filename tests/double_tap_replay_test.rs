mod common;

use subtr_actor::{GoalTagKind, StatsTimelineCollector};

const THIRD_GOAL_DOUBLE_TAP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";

#[test]
fn tags_colonelpanic8_third_goal_as_double_tap() {
    let replay = common::parse_replay(THIRD_GOAL_DOUBLE_TAP_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("failed to collect stats timeline for double tap replay");

    assert!(
        timeline.events.double_tap.iter().any(|event| {
            (event.backboard_time - 174.0366).abs() < 0.05 && (event.time - 174.4810).abs() < 0.05
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
                    } => (start_time - 174.0366).abs() < 0.05 && (end_time - 174.4810).abs() < 0.05,
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
