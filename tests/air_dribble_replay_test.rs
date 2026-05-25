mod common;

use subtr_actor::StatsTimelineCollector;

const AIR_DRIBBLE_GOAL_MOUTH_REPLAY: &str = "assets/air-dribble-goal-mouth-2026-05-24.replay";

#[test]
fn detects_colonelpanic8_air_dribble_goal_roughly_at_goal_sequence() {
    let replay = common::parse_replay(AIR_DRIBBLE_GOAL_MOUTH_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("failed to collect stats timeline for air dribble goal replay");

    let event = common::assert_mechanic_event_roughly_at(
        &timeline,
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
}
