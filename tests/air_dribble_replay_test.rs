mod common;

use subtr_actor::{EventPayload, GoalTagKind, StatsTimelineEventCollector};

const AIR_DRIBBLE_GOAL_MOUTH_REPLAY: &str = "assets/air-dribble-goal-mouth-2026-05-24.replay";

#[test]
fn detects_air_dribble_goal_and_rejects_unrelated_half_volley_tag() {
    let replay = common::parse_replay(AIR_DRIBBLE_GOAL_MOUTH_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("failed to collect stats timeline for air dribble goal replay");

    let event = common::assert_mechanic_event_roughly_at_in_meta(
        &timeline.replay_meta,
        &timeline.events.events,
        "air_dribble",
        "colonelpanic8",
        56.37,
        59.11,
        0.25,
    );

    assert_eq!(
        common::mechanic_event_text_property(event, "origin"),
        Some("ground_to_air")
    );
    assert!(
        common::mechanic_event_unsigned_property(event, "touch_count").unwrap_or(0) >= 3,
        "expected colonelpanic8 air dribble to include at least 3 touches"
    );

    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });

    assert!(
        goal_context.len() >= 6,
        "expected at least six goals in replay; got {goal_context:?}"
    );
    assert!(
        !goal_context.get(5).is_some_and(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::HalfVolleyGoal)),
        "expected sixth goal not to be tagged as a half volley; got {goal_context:?}"
    );
}
