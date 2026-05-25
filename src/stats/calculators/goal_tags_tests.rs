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
        ball_air_time_before_goal: None,
        goal_buildup: GoalBuildupKind::Other,
        scorer_last_touch: Some(scorer_touch(touch_position, players.clone())),
        players,
    }
}

fn tag_kinds(events: &[GoalTagEvent]) -> Vec<GoalTagKind> {
    let mut kinds: Vec<_> = events.iter().map(|event| event.kind).collect();
    kinds.sort_by_key(|kind| format!("{kind:?}"));
    kinds
}

fn has_modifier(event: &GoalTagEvent, modifier: GoalTagModifier) -> bool {
    event.modifiers.contains(&modifier)
}

fn all_goal_tag_events(goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
    let aerial = AerialGoalCalculator::new();
    let high_aerial = HighAerialGoalCalculator::new();
    let long_distance = LongDistanceGoalCalculator::new();
    let own_half = OwnHalfGoalCalculator::new();
    let empty_net = EmptyNetGoalCalculator::new();
    let counter_attack = CounterAttackGoalCalculator::new();
    let aerial_events = aerial.tag_goals(goals);
    let high_aerial_events = high_aerial.tag_goals(goals);
    let long_distance_events = long_distance.tag_goals(goals);
    let own_half_events = own_half.tag_goals(goals);
    let empty_net_events = empty_net.tag_goals(goals);
    let counter_attack_events = counter_attack.tag_goals(goals);

    combined_goal_tag_events(&[
        &aerial_events,
        &high_aerial_events,
        &long_distance_events,
        &own_half_events,
        &empty_net_events,
        &counter_attack_events,
    ])
}

fn flick_event(time: f32, frame: usize, player: PlayerId) -> FlickEvent {
    FlickEvent {
        time,
        frame,
        player,
        is_team_0: true,
        dodge_time: time - 0.1,
        dodge_frame: frame.saturating_sub(1),
        time_since_dodge: 0.1,
        setup_start_time: time - 0.3,
        setup_start_frame: frame.saturating_sub(3),
        setup_duration: 0.3,
        setup_touch_count: 2,
        average_horizontal_gap: 40.0,
        average_vertical_gap: 80.0,
        ball_speed_change: 600.0,
        ball_impulse: [0.0, 600.0, 0.0],
        impulse_away_alignment: 0.8,
        vertical_impulse: 0.0,
        confidence: 0.82,
    }
}

fn one_timer_event(time: f32, frame: usize, player: PlayerId) -> OneTimerEvent {
    OneTimerEvent {
        time,
        frame,
        player,
        passer: player_id(2),
        is_team_0: true,
        pass_start_time: time - 0.8,
        pass_start_frame: frame.saturating_sub(8),
        pass_duration: 0.8,
        pass_travel_distance: 1200.0,
        pass_advance_distance: 900.0,
        ball_speed: 1800.0,
        goal_alignment: 0.9,
    }
}

fn air_dribble_event(
    start_time: f32,
    end_time: f32,
    player: PlayerId,
    kind: BallCarryKind,
) -> BallCarryEvent {
    BallCarryEvent {
        player_id: player,
        is_team_0: true,
        kind,
        start_frame: (start_time * 10.0) as usize,
        end_frame: (end_time * 10.0) as usize,
        start_time,
        end_time,
        duration: end_time - start_time,
        straight_line_distance: 900.0,
        path_distance: 1000.0,
        average_horizontal_gap: 80.0,
        average_vertical_gap: 120.0,
        average_speed: 700.0,
        touch_count: 0,
        air_touch_count: 0,
        air_dribble_origin: (kind == BallCarryKind::AirDribble)
            .then_some(AirDribbleOrigin::GroundToAir),
    }
}

fn dodge_refreshed_event(time: f32, frame: usize, player: PlayerId) -> DodgeRefreshedEvent {
    DodgeRefreshedEvent {
        time,
        frame,
        player,
        is_team_0: true,
        counter_value: 1,
    }
}

fn half_volley_event(time: f32, frame: usize, player: PlayerId) -> HalfVolleyEvent {
    HalfVolleyEvent {
        time,
        frame,
        player,
        is_team_0: true,
        bounce_time: time - 0.2,
        bounce_frame: frame.saturating_sub(2),
        bounce_to_touch_seconds: 0.2,
        ball_speed: 1600.0,
        goal_alignment: 0.8,
    }
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
fn own_half_goal_rejects_stale_scorer_touch() {
    let mut goal = goal_with_touch(true, position(0.0, -100.0, 120.0), Vec::new());
    goal.scorer_last_touch.as_mut().unwrap().time = 1.0;

    let own_half_events = OwnHalfGoalCalculator::new().tag_goals(&[goal.clone()]);
    let long_distance_events = LongDistanceGoalCalculator::new().tag_goals(&[goal]);

    assert!(own_half_events.is_empty());
    assert_eq!(
        tag_kinds(&long_distance_events),
        vec![GoalTagKind::LongDistanceGoal]
    );
}

#[test]
fn own_half_goal_uses_scoring_team_orientation() {
    let team_zero_own_half = goal_with_touch(true, position(0.0, -100.0, 120.0), Vec::new());
    let team_zero_opposing_half = goal_with_touch(true, position(0.0, 100.0, 120.0), Vec::new());
    let team_one_own_half = goal_with_touch(false, position(0.0, 100.0, 120.0), Vec::new());
    let team_one_opposing_half = goal_with_touch(false, position(0.0, -100.0, 120.0), Vec::new());

    let calculator = OwnHalfGoalCalculator::new();

    assert_eq!(
        tag_kinds(&calculator.tag_goals(&[team_zero_own_half])),
        vec![GoalTagKind::OwnHalfGoal]
    );
    assert!(calculator.tag_goals(&[team_zero_opposing_half]).is_empty());
    assert_eq!(
        tag_kinds(&calculator.tag_goals(&[team_one_own_half])),
        vec![GoalTagKind::OwnHalfGoal]
    );
    assert!(calculator.tag_goals(&[team_one_opposing_half]).is_empty());
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

#[test]
fn counter_attack_goal_tags_goal_with_counter_attack_buildup() {
    let mut goal = goal_with_touch(true, position(0.0, 1800.0, 120.0), Vec::new());
    goal.goal_buildup = GoalBuildupKind::CounterAttack;

    let events = CounterAttackGoalCalculator::new().tag_goals(&[goal]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::CounterAttackGoal]);
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::GoalBuildup));
}

#[test]
fn counter_attack_goal_rejects_other_buildup() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 120.0), Vec::new());

    let events = CounterAttackGoalCalculator::new().tag_goals(&[goal]);

    assert!(events.is_empty());
}

#[test]
fn flick_goal_tags_matching_scorer_flick_before_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 180.0), Vec::new());
    let events =
        FlickGoalCalculator::new().tag_goals(&[goal], &[flick_event(9.3, 93, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlickGoal]);
    assert_eq!(events[0].confidence, 0.82);
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::Flick));
}

#[test]
fn flick_goal_rejects_stale_flicks() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 180.0), Vec::new());
    let events =
        FlickGoalCalculator::new().tag_goals(&[goal], &[flick_event(6.5, 65, player_id(1))]);

    assert!(events.is_empty());
}

#[test]
fn flick_goal_can_be_created_by_scoring_teammate() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 180.0), Vec::new());
    let events =
        FlickGoalCalculator::new().tag_goals(&[goal], &[flick_event(8.8, 88, player_id(2))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlickGoal]);
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert_eq!(
        events[0]
            .evidence
            .iter()
            .find(|evidence| evidence.kind == GoalTagEvidenceKind::Flick)
            .and_then(|evidence| evidence.player.as_ref()),
        Some(&player_id(2))
    );
}

#[test]
fn one_timer_goal_tags_matching_one_timer_before_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 2000.0, 120.0), Vec::new());
    let events =
        OneTimerGoalCalculator::new().tag_goals(&[goal], &[one_timer_event(9.4, 94, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::OneTimerGoal]);
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::OneTimer));
}

#[test]
fn air_dribble_goal_tags_air_dribble_control_that_reaches_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 600.0), Vec::new());
    let events = AirDribbleGoalCalculator::new().tag_goals(
        &[goal],
        &[air_dribble_event(
            8.0,
            9.2,
            player_id(1),
            BallCarryKind::AirDribble,
        )],
    );

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::AirDribbleGoal]);
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::AirDribble));
}

#[test]
fn air_dribble_goal_can_be_created_by_scoring_teammate() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 600.0), Vec::new());
    let events = AirDribbleGoalCalculator::new().tag_goals(
        &[goal],
        &[air_dribble_event(
            7.8,
            9.0,
            player_id(2),
            BallCarryKind::AirDribble,
        )],
    );

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::AirDribbleGoal]);
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
}

#[test]
fn air_dribble_goal_rejects_ground_carries() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 600.0), Vec::new());
    let events = AirDribbleGoalCalculator::new().tag_goals(
        &[goal],
        &[air_dribble_event(
            8.0,
            9.4,
            player_id(1),
            BallCarryKind::Carry,
        )],
    );

    assert!(events.is_empty());
}

#[test]
fn flip_reset_goal_tags_matching_on_ball_reset_before_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 2400.0, 700.0), Vec::new());
    let events = FlipResetGoalCalculator::new()
        .tag_goals(&[goal], &[dodge_refreshed_event(7.0, 70, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlipResetGoal]);
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::FlipReset));
}

#[test]
fn half_volley_goal_tags_scorer_last_touch_after_floor_bounce() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let calculator = HalfVolleyGoalCalculator::new();
    let half_volleys = vec![half_volley_event(9.5, 95, player_id(1))];

    let events = calculator.tag_goals(&[goal], &half_volleys);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::HalfVolleyGoal]);
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::HalfVolley));
}

#[test]
fn half_volley_goal_requires_the_scorer_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let calculator = HalfVolleyGoalCalculator::new();
    let half_volleys = vec![half_volley_event(9.4, 94, player_id(1))];

    let events = calculator.tag_goals(&[goal], &half_volleys);

    assert!(events.is_empty());
}

#[test]
fn half_volley_goal_rejects_stale_touches() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let calculator = HalfVolleyGoalCalculator::with_config(HalfVolleyGoalCalculatorConfig {
        max_touch_to_goal_seconds: 0.3,
        ..HalfVolleyGoalCalculatorConfig::default()
    });
    let half_volleys = vec![half_volley_event(9.5, 95, player_id(1))];

    let events = calculator.tag_goals(&[goal], &half_volleys);

    assert!(events.is_empty());
}
