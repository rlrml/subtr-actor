use super::*;

fn player_id(value: u64) -> PlayerId {
    boxcars::RemoteId::Steam(value)
}

fn position(x: f32, y: f32, z: f32) -> GoalContextPosition {
    GoalContextPosition { x, y, z }
}

fn scorer_touch(
    ball_position: GoalContextPosition,
    players: Vec<GoalPlayerContext>,
) -> GoalTouchContext {
    GoalTouchContext {
        time: 9.5,
        frame: 95,
        player: player_id(1),
        is_team_0: true,
        ball_position: Some(ball_position),
        player_position: Some(position(0.0, ball_position.y, 20.0)),
        players,
    }
}

fn player_context(
    player: PlayerId,
    is_team_0: bool,
    position: GoalContextPosition,
) -> GoalPlayerContext {
    GoalPlayerContext {
        player,
        is_team_0,
        position: Some(position),
        boost_amount: None,
        average_boost_in_leadup: None,
        min_boost_in_leadup: None,
        is_most_back: false,
    }
}

fn goal_with_touch(
    scoring_team_is_team_0: bool,
    touch_position: GoalContextPosition,
    players: Vec<GoalPlayerContext>,
) -> GoalContextEvent {
    GoalContextEvent {
        time: 10.0,
        frame: 100,
        scoring_team_is_team_0,
        scorer: Some(player_id(1)),
        scoring_team_most_back_player: None,
        defending_team_most_back_player: None,
        ball_position: Some(touch_position),
        scorer_last_touch: Some(scorer_touch(touch_position, players.clone())),
        players,
    }
}

fn tag_kinds(events: &[GoalTagEvent]) -> Vec<GoalTagKind> {
    let mut kinds: Vec<_> = events.iter().map(|event| event.kind).collect();
    kinds.sort_by_key(|kind| format!("{kind:?}"));
    kinds
}

fn all_goal_tag_events(goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
    let aerial = AerialGoalCalculator::new();
    let high_aerial = HighAerialGoalCalculator::new();
    let long_distance = LongDistanceGoalCalculator::new();
    let own_half = OwnHalfGoalCalculator::new();
    let empty_net = EmptyNetGoalCalculator::new();
    let aerial_events = aerial.tag_goals(goals);
    let high_aerial_events = high_aerial.tag_goals(goals);
    let long_distance_events = long_distance.tag_goals(goals);
    let own_half_events = own_half.tag_goals(goals);
    let empty_net_events = empty_net.tag_goals(goals);

    combined_goal_tag_events(&[
        &aerial_events,
        &high_aerial_events,
        &long_distance_events,
        &own_half_events,
        &empty_net_events,
    ])
}

#[test]
fn high_aerial_goal_also_gets_aerial_goal_tag() {
    let goal = goal_with_touch(true, position(0.0, 1500.0, 900.0), Vec::new());

    let events = all_goal_tag_events(&[goal]);

    assert_eq!(
        tag_kinds(&events),
        vec![GoalTagKind::AerialGoal, GoalTagKind::HighAerialGoal]
    );
}

#[test]
fn own_half_goal_also_gets_long_distance_goal_tag() {
    let goal = goal_with_touch(true, position(0.0, -100.0, 120.0), Vec::new());

    let events = all_goal_tag_events(&[goal]);

    assert_eq!(
        tag_kinds(&events),
        vec![GoalTagKind::LongDistanceGoal, GoalTagKind::OwnHalfGoal]
    );
}

#[test]
fn long_distance_goal_does_not_require_own_half_touch() {
    let goal = goal_with_touch(true, position(0.0, 800.0, 120.0), Vec::new());

    let events = all_goal_tag_events(&[goal]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::LongDistanceGoal]);
}

#[test]
fn empty_net_goal_requires_defenders_to_be_clearly_behind_the_play() {
    let goal = goal_with_touch(
        true,
        position(0.0, 2500.0, 120.0),
        vec![
            player_context(player_id(1), true, position(0.0, 2500.0, 20.0)),
            player_context(player_id(2), false, position(0.0, 1200.0, 20.0)),
            player_context(player_id(3), false, position(800.0, 800.0, 20.0)),
        ],
    );

    let events = all_goal_tag_events(&[goal]);

    assert!(tag_kinds(&events).contains(&GoalTagKind::EmptyNetGoal));
}

#[test]
fn empty_net_goal_rejects_barely_behind_defenders() {
    let goal = goal_with_touch(
        true,
        position(0.0, 2500.0, 120.0),
        vec![player_context(
            player_id(2),
            false,
            position(0.0, 2000.0, 20.0),
        )],
    );

    let events = all_goal_tag_events(&[goal]);

    assert!(!tag_kinds(&events).contains(&GoalTagKind::EmptyNetGoal));
}

#[test]
fn empty_net_goal_rejects_goal_mouth_scrambles() {
    let goal = goal_with_touch(
        true,
        position(0.0, 3900.0, 120.0),
        vec![player_context(
            player_id(2),
            false,
            position(0.0, 2000.0, 20.0),
        )],
    );

    let events = all_goal_tag_events(&[goal]);

    assert!(!tag_kinds(&events).contains(&GoalTagKind::EmptyNetGoal));
}
