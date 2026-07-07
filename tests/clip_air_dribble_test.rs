//! Clip-based variant of `air_dribble_replay_test.rs`.
//!
//! Clips a window around the reviewed air dribble (and the goal it produces)
//! and asserts the same detections on the clip instead of processing the whole
//! replay. Goal indices shift inside a clip (earlier goals are outside the
//! window), so the goal assertion matches the goal *inside* the clip rather
//! than the source replay's sixth goal.

mod common;

use subtr_actor::{
    EventPayload, GoalTagKind, StatsTimelineEventCollector, clip_replay_around_times,
};

const AIR_DRIBBLE_GOAL_MOUTH_REPLAY: &str = "assets/air-dribble-goal-mouth-2026-05-24.replay";

// Reviewed ground truth: the dribbler's earlier ground pop at ~53.7s falls all
// the way back to resting height (ball Z bottoms at ~95uu around 56.0s) before
// the real air dribble opens from the ground with the touch at ~56.23s, after
// which the ball stays airborne through the remaining touches into the goal.
const AIR_DRIBBLE_START_TIME: f32 = 56.23;
const AIR_DRIBBLE_END_TIME: f32 = 59.02;

#[test]
fn clip_detects_air_dribble_goal_and_rejects_unrelated_half_volley_tag() {
    let replay = common::parse_replay(AIR_DRIBBLE_GOAL_MOUTH_REPLAY);
    // Tail long enough to include the goal the dribble scores plus the goal
    // celebration state that closes out goal context tagging.
    let clip = clip_replay_around_times(
        &replay,
        AIR_DRIBBLE_START_TIME,
        AIR_DRIBBLE_END_TIME,
        90,
        180,
    )
    .expect("clip should build");
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("failed to collect stats timeline for air dribble clip");

    let event = common::assert_mechanic_event_roughly_at_in_meta(
        &timeline.replay_meta,
        &timeline.events.events,
        "air_dribble",
        "colonelpanic8",
        AIR_DRIBBLE_START_TIME,
        AIR_DRIBBLE_END_TIME,
        0.25,
    );

    let EventPayload::BallCarry(air_dribble) = &event.payload else {
        panic!("expected air_dribble event payload to be BallCarry: {event:#?}");
    };
    assert_eq!(
        air_dribble
            .air_dribble_origin
            .map(|origin| origin.as_label_value()),
        Some("ground_to_air")
    );
    assert!(
        air_dribble.touch_count >= 3,
        "expected colonelpanic8 air dribble to include at least 3 touches"
    );

    let goal_context = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::GoalContext(event) => Some(event),
        _ => None,
    });

    // The clip contains exactly the goal the air dribble scores (the source
    // replay's sixth); it must not be tagged as a half volley.
    assert!(
        !goal_context.is_empty(),
        "expected the air dribble goal inside the clip; got no goal context events"
    );
    assert!(
        !goal_context.iter().any(|goal| goal
            .tags
            .iter()
            .any(|tag| tag.kind() == GoalTagKind::HalfVolleyGoal)),
        "expected the air dribble goal not to be tagged as a half volley; got {goal_context:?}"
    );
}
