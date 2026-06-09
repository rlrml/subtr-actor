use super::*;
use crate::stats::calculators::rotation::{PlayDepthState, RoleState};

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
        touch_id: None,
        time: 9.5,
        frame: 95,
        player: player_id(1),
        is_team_0: true,
        ball_position: Some(ball_position),
        ball_speed_after_touch: None,
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
        ball_speed_at_goal: None,
        ball_air_time_before_goal: None,
        pressure_duration_before_goal: None,
        time_after_kickoff: None,
        goal_buildup: GoalBuildupKind::Other,
        scorer_last_touch: Some(scorer_touch(touch_position, players.clone())),
        players,
        tags: Vec::new(),
    }
}

fn tag_kinds(events: &[GoalTagAssignment]) -> Vec<GoalTagKind> {
    let mut kinds: Vec<_> = events.iter().map(|event| event.tag.kind()).collect();
    kinds.sort_by_key(|kind| format!("{kind:?}"));
    kinds
}

fn has_modifier(event: &GoalTagAssignment, modifier: GoalTagModifier) -> bool {
    event.tag.metadata().modifiers.contains(&modifier)
}

fn performer(event: &GoalTagAssignment) -> Option<GoalTagPerformer> {
    event.tag.metadata().performer
}

fn all_goal_tag_events(goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
    let aerial = AerialGoalCalculator::new();
    let high_aerial = HighAerialGoalCalculator::new();
    let long_distance = LongDistanceGoalCalculator::new();
    let own_half = OwnHalfGoalCalculator::new();
    let empty_net = EmptyNetGoalCalculator::new();
    let counter_attack = CounterAttackGoalCalculator::new();
    let sustained_pressure = SustainedPressureGoalCalculator::new();
    let aerial_events = aerial.tag_goals(goals);
    let high_aerial_events = high_aerial.tag_goals(goals);
    let long_distance_events = long_distance.tag_goals(goals);
    let own_half_events = own_half.tag_goals(goals);
    let empty_net_events = empty_net.tag_goals(goals);
    let counter_attack_events = counter_attack.tag_goals(goals);
    let sustained_pressure_events = sustained_pressure.tag_goals(goals);

    combined_goal_tag_assignments(&[
        &aerial_events,
        &high_aerial_events,
        &long_distance_events,
        &own_half_events,
        &empty_net_events,
        &counter_attack_events,
        &sustained_pressure_events,
    ])
}

fn flick_event(time: f32, frame: usize, player: PlayerId) -> FlickEvent {
    FlickEvent {
        time,
        frame,
        sample_time: time,
        sample_frame: frame,
        player,
        player_position: None,
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
        kind: "other".to_owned(),
        local_ball_position: [60.0, 0.0, 95.0],
        local_ball_impulse: [0.0, 600.0, 0.0],
        backflip_pitch_rate: 0.0,
        rotation_under_ball_degrees: 0.0,
        setup_rotation_degrees: 0.0,
        setup_rotation_direction: "unknown".to_owned(),
        confidence: 0.82,
    }
}

fn ceiling_shot_event(time: f32, frame: usize, player: PlayerId) -> CeilingShotEvent {
    CeilingShotEvent {
        time,
        frame,
        player,
        player_position: None,
        is_team_0: true,
        ceiling_contact_time: time - 0.8,
        ceiling_contact_frame: frame.saturating_sub(8),
        time_since_ceiling_contact: 0.8,
        ceiling_contact_position: [0.0, 900.0, 2044.0],
        touch_position: [0.0, 1800.0, 820.0],
        local_ball_position: [0.0, 120.0, -60.0],
        separation_from_ceiling: 600.0,
        roof_alignment: 0.9,
        forward_alignment: 0.8,
        forward_approach_speed: 1100.0,
        ball_speed_change: 700.0,
        confidence: 0.84,
    }
}

fn one_timer_event(time: f32, frame: usize, player: PlayerId) -> OneTimerEvent {
    OneTimerEvent {
        time,
        frame,
        player,
        player_position: None,
        passer: player_id(2),
        passer_position: None,
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

fn pass_event(time: f32, frame: usize, passer: PlayerId, receiver: PlayerId) -> PassEvent {
    PassEvent {
        time,
        frame,
        sample_time: time,
        sample_frame: frame,
        passer,
        passer_position: None,
        receiver,
        receiver_position: None,
        is_team_0: true,
        start_time: time - 1.0,
        start_frame: frame.saturating_sub(10),
        duration: 1.0,
        ball_travel_distance: 1200.0,
        ball_advance_distance: 800.0,
        pass_kind: PassKind::Direct,
    }
}

fn double_tap_event(time: f32, frame: usize, player: PlayerId) -> DoubleTapEvent {
    DoubleTapEvent {
        time,
        frame,
        player,
        player_position: None,
        is_team_0: true,
        backboard_time: time - 0.4,
        backboard_frame: frame.saturating_sub(4),
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
        start_position: [0.0, 0.0, 0.0],
        end_position: [0.0, 0.0, 0.0],
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

fn confirmed_flip_reset_event(
    time: f32,
    frame: usize,
    player: PlayerId,
) -> ConfirmedFlipResetEvent {
    ConfirmedFlipResetEvent {
        time,
        frame,
        reset_time: time - 0.5,
        reset_frame: frame.saturating_sub(5),
        player,
        player_position: None,
        is_team_0: true,
        counter_value: 1,
        time_since_reset: 0.5,
    }
}

fn touch_classification_event(
    time: f32,
    frame: usize,
    player: PlayerId,
    dodge_state: &str,
) -> TouchClassificationEvent {
    TouchClassificationEvent {
        touch_id: None,
        time,
        frame,
        sample_time: time,
        sample_frame: frame,
        player,
        player_position: Some([0.0, 2200.0, 60.0]),
        is_team_0: true,
        kind: "medium_hit".to_owned(),
        height_band: "ground".to_owned(),
        surface: "ground".to_owned(),
        dodge_state: dodge_state.to_owned(),
        intention: "neutral".to_owned(),
        first_touch: false,
        contested: false,
        role: RoleState::Unknown,
        play_depth: PlayDepthState::Unknown,
        ball_speed_change: 800.0,
        ball_movement: None,
    }
}

fn bump_event(time: f32, frame: usize, initiator: PlayerId, victim: PlayerId) -> BumpEvent {
    BumpEvent {
        time,
        frame,
        initiator,
        victim,
        initiator_is_team_0: true,
        victim_is_team_0: false,
        is_team_bump: false,
        strength: 800.0,
        confidence: 0.76,
        contact_distance: 100.0,
        closing_speed: 900.0,
        victim_impulse: 600.0,
        initiator_position: [10.0, 1200.0, 20.0],
        victim_position: [30.0, 1180.0, 20.0],
    }
}

fn demo_event(time: f32, frame: usize, attacker: PlayerId) -> TimelineEvent {
    TimelineEvent {
        time,
        frame: Some(frame),
        kind: TimelineEventKind::Kill,
        player_id: Some(attacker),
        player_position: Some([10.0, 1200.0, 20.0]),
        is_team_0: Some(true),
    }
}

fn half_volley_event(time: f32, frame: usize, player: PlayerId) -> HalfVolleyEvent {
    HalfVolleyEvent {
        time,
        frame,
        sample_time: time,
        sample_frame: frame,
        player,
        player_position: None,
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
        .tag
        .metadata()
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
fn sustained_pressure_goal_tags_goal_with_sustained_pressure_buildup() {
    let mut goal = goal_with_touch(true, position(0.0, 1800.0, 120.0), Vec::new());
    goal.goal_buildup = GoalBuildupKind::SustainedPressure;

    let events = SustainedPressureGoalCalculator::new().tag_goals(&[goal]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::SustainedPressureGoal]);
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::GoalBuildup));
}

#[test]
fn sustained_pressure_goal_rejects_other_buildup() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 120.0), Vec::new());

    let events = SustainedPressureGoalCalculator::new().tag_goals(&[goal]);

    assert!(events.is_empty());
}

fn kickoff_event_for_goal(
    scoring_team_is_team_0: Option<bool>,
    first_touch_time: Option<f32>,
    time_to_goal: Option<f32>,
    kickoff_goal: bool,
) -> KickoffEvent {
    KickoffEvent {
        start_time: first_touch_time.unwrap_or(0.0) - 1.0,
        start_frame: 0,
        end_time: first_touch_time.unwrap_or(0.0) + 2.0,
        end_frame: 30,
        live_action_start_time: None,
        live_action_start_frame: None,
        movement_start_time: 0.0,
        movement_start_frame: 0,
        kickoff_type: KickoffType::Center,
        kickoff_direction: KickoffDirection::Center,
        first_touch_time,
        first_touch_frame: None,
        first_touch_team_is_team_0: None,
        first_touch_player: None,
        first_touch_id: None,
        first_touch_ball_position: None,
        first_touch_ball_abs_x: None,
        first_touch_ball_height: None,
        first_touch_ball_velocity: None,
        team_zero_taker_touch_time: None,
        team_zero_taker_touch_frame: None,
        team_one_taker_touch_time: None,
        team_one_taker_touch_frame: None,
        taker_touch_delay_seconds: None,
        exit_velocity: None,
        exit_speed: None,
        exit_y_velocity: None,
        first_follow_up_touch_time: None,
        first_follow_up_touch_frame: None,
        first_follow_up_touch_team_is_team_0: None,
        first_follow_up_touch_player: None,
        outcome: KickoffOutcome::Unknown,
        winning_team_is_team_0: None,
        win_strength: None,
        win_strength_band: KickoffWinStrengthBand::Unknown,
        kickoff_possession_outcome: KickoffPossessionOutcome::Contested,
        kickoff_possession_team_is_team_0: None,
        kickoff_goal,
        scoring_team_is_team_0,
        time_to_goal,
        team_zero_taker: None,
        team_one_taker: None,
        team_zero_non_takers: Vec::new(),
        team_one_non_takers: Vec::new(),
    }
}

#[test]
fn kickoff_goal_tags_goals_attributed_by_a_kickoff_event() {
    // goal_with_touch builds a goal at t=10.0; the kickoff attributed it as
    // first touch at 7.3 + 2.7 to goal.
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let kickoff_event = kickoff_event_for_goal(Some(true), Some(7.3), Some(2.7), true);
    let calculator = KickoffGoalCalculator::new();

    let events = calculator.tag_goals(&[goal], &[kickoff_event]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::KickoffGoal]);
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::GoalContext));
}

#[test]
fn kickoff_goal_rejects_goals_without_a_matching_kickoff_attribution() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let unattributed = kickoff_event_for_goal(Some(true), Some(7.3), None, false);
    let different_goal_time = kickoff_event_for_goal(Some(true), Some(1.0), Some(2.0), true);
    let different_team = kickoff_event_for_goal(Some(false), Some(7.3), Some(2.7), true);
    let calculator = KickoffGoalCalculator::new();

    let events = calculator.tag_goals(
        &[goal],
        &[unattributed, different_goal_time, different_team],
    );

    assert!(events.is_empty());
}

#[test]
fn flick_goal_tags_matching_scorer_flick_before_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 1800.0, 180.0), Vec::new());
    let events =
        FlickGoalCalculator::new().tag_goals(&[goal], &[flick_event(9.3, 93, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlickGoal]);
    assert_eq!(events[0].tag.metadata().confidence, 0.82);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
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
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert_eq!(
        events[0]
            .tag
            .metadata()
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
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::OneTimer));
}

#[test]
fn passing_goal_tags_pass_received_by_scorer_on_last_touch() {
    let goal = goal_with_touch(true, position(0.0, 2000.0, 120.0), Vec::new());
    let events = PassingGoalCalculator::new()
        .tag_goals(&[goal], &[pass_event(9.5, 95, player_id(2), player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::PassingGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert_eq!(
        events[0]
            .tag
            .metadata()
            .evidence
            .iter()
            .find(|evidence| evidence.kind == GoalTagEvidenceKind::Pass)
            .and_then(|evidence| evidence.player.as_ref()),
        Some(&player_id(2))
    );
}

#[test]
fn passing_goal_rejects_pass_not_received_by_scorer() {
    let goal = goal_with_touch(true, position(0.0, 2000.0, 120.0), Vec::new());
    let events = PassingGoalCalculator::new()
        .tag_goals(&[goal], &[pass_event(9.5, 95, player_id(2), player_id(3))]);

    assert!(events.is_empty());
}

#[test]
fn ceiling_shot_goal_tags_matching_ceiling_shot_before_goal() {
    let goal = goal_with_touch(true, position(0.0, 2400.0, 800.0), Vec::new());
    let events = CeilingShotGoalCalculator::new()
        .tag_goals(&[goal], &[ceiling_shot_event(9.4, 94, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::CeilingShotGoal]);
    assert_eq!(events[0].tag.metadata().confidence, 0.84);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::CeilingShot));
}

#[test]
fn ceiling_shot_goal_can_be_created_by_scoring_teammate() {
    let goal = goal_with_touch(true, position(0.0, 2400.0, 800.0), Vec::new());
    let events = CeilingShotGoalCalculator::new()
        .tag_goals(&[goal], &[ceiling_shot_event(9.4, 94, player_id(2))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::CeilingShotGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
}

#[test]
fn ceiling_shot_goal_rejects_stale_events() {
    let goal = goal_with_touch(true, position(0.0, 2400.0, 800.0), Vec::new());
    let calculator = CeilingShotGoalCalculator::with_config(CeilingShotGoalCalculatorConfig {
        max_event_to_goal_seconds: 0.3,
    });
    let events = calculator.tag_goals(&[goal], &[ceiling_shot_event(9.4, 94, player_id(1))]);

    assert!(events.is_empty());
}

#[test]
fn double_tap_goal_tags_matching_double_tap_before_goal() {
    let goal = goal_with_touch(true, position(0.0, 2800.0, 500.0), Vec::new());
    let events = DoubleTapGoalCalculator::new()
        .tag_goals(&[goal], &[double_tap_event(9.4, 94, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::DoubleTapGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::DoubleTap));
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
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
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
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
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
    let events = FlipResetGoalCalculator::new().tag_goals(
        &[goal],
        &[confirmed_flip_reset_event(7.0, 70, player_id(1))],
    );

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlipResetGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::FlipReset));
}

#[test]
fn flip_reset_goal_can_be_created_by_scoring_teammate() {
    let goal = goal_with_touch(true, position(0.0, 2400.0, 700.0), Vec::new());
    let events = FlipResetGoalCalculator::new().tag_goals(
        &[goal],
        &[confirmed_flip_reset_event(7.0, 70, player_id(2))],
    );

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlipResetGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
}

#[test]
fn flip_into_ball_goal_tags_dodge_scoring_touch() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let touches = vec![touch_classification_event(9.5, 95, player_id(1), "dodge")];

    let events = FlipIntoBallGoalCalculator::new().tag_goals(&[goal], &touches);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlipIntoBallGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert!(events[0]
        .tag
        .metadata()
        .evidence
        .iter()
        .any(|evidence| evidence.kind == GoalTagEvidenceKind::FlipIntoBall));
    assert_eq!(
        events[0].tag.metadata().related_events,
        vec![GoalTagEventRef {
            stream: GoalTagEventStream::Touch,
            index: 0,
        }]
    );
}

#[test]
fn flip_into_ball_goal_rejects_non_dodge_scoring_touch() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let touches = vec![touch_classification_event(
        9.5,
        95,
        player_id(1),
        "no_dodge",
    )];

    let events = FlipIntoBallGoalCalculator::new().tag_goals(&[goal], &touches);

    assert!(events.is_empty());
}

#[test]
fn flip_into_ball_goal_requires_dodge_on_the_scoring_touch_itself() {
    // A flip-reset-style refresh: an earlier dodge touch by the scorer, but
    // the scoring touch itself was not a dodge contact.
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let touches = vec![
        touch_classification_event(8.0, 80, player_id(1), "dodge"),
        touch_classification_event(9.5, 95, player_id(1), "no_dodge"),
    ];

    let events = FlipIntoBallGoalCalculator::new().tag_goals(&[goal], &touches);

    assert!(events.is_empty());
}

#[test]
fn flip_into_ball_goal_requires_the_scorer_touch() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let touches = vec![touch_classification_event(9.5, 95, player_id(2), "dodge")];

    let events = FlipIntoBallGoalCalculator::new().tag_goals(&[goal], &touches);

    assert!(events.is_empty());
}

#[test]
fn flip_into_ball_goal_rejects_stale_touches() {
    let goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    let calculator = FlipIntoBallGoalCalculator::with_config(FlipIntoBallGoalCalculatorConfig {
        max_touch_to_goal_seconds: 0.3,
    });
    let touches = vec![touch_classification_event(9.5, 95, player_id(1), "dodge")];

    let events = calculator.tag_goals(&[goal], &touches);

    assert!(events.is_empty());
}

#[test]
fn flip_into_ball_goal_joins_by_touch_id_when_present() {
    let mut goal = goal_with_touch(true, position(0.0, 2200.0, 130.0), Vec::new());
    goal.scorer_last_touch.as_mut().unwrap().touch_id = Some(7);

    // Same player + frame as the scoring touch, but a different identity:
    // an id-bearing candidate that is not the scoring touch must not match.
    let mut other_touch = touch_classification_event(9.5, 95, player_id(1), "dodge");
    other_touch.touch_id = Some(6);
    assert!(FlipIntoBallGoalCalculator::new()
        .tag_goals(std::slice::from_ref(&goal), &[other_touch.clone()])
        .is_empty());

    // The candidate carrying the matching id is the scoring touch.
    let mut scoring_touch = other_touch;
    scoring_touch.touch_id = Some(7);
    let events =
        FlipIntoBallGoalCalculator::new().tag_goals(std::slice::from_ref(&goal), &[scoring_touch]);
    assert_eq!(tag_kinds(&events), vec![GoalTagKind::FlipIntoBallGoal]);
}

#[test]
fn bump_goal_can_be_created_by_non_scorer_teammate() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let events = BumpGoalCalculator::new()
        .tag_goals(&[goal], &[bump_event(9.1, 91, player_id(2), player_id(3))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::BumpGoal]);
    assert_eq!(events[0].tag.metadata().confidence, 0.76);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert_eq!(
        events[0]
            .tag
            .metadata()
            .evidence
            .iter()
            .find(|evidence| evidence.kind == GoalTagEvidenceKind::Bump)
            .and_then(|evidence| evidence.player.as_ref()),
        Some(&player_id(2))
    );
}

#[test]
fn bump_goal_marks_by_scorer_when_scorer_inflicts_bump() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let events = BumpGoalCalculator::new()
        .tag_goals(&[goal], &[bump_event(9.1, 91, player_id(1), player_id(3))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::BumpGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
}

#[test]
fn bump_goal_rejects_team_bumps_and_opponent_bumps() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let mut team_bump = bump_event(9.1, 91, player_id(2), player_id(4));
    team_bump.victim_is_team_0 = true;
    team_bump.is_team_bump = true;
    let mut opponent_bump = bump_event(9.1, 91, player_id(3), player_id(2));
    opponent_bump.initiator_is_team_0 = false;
    opponent_bump.victim_is_team_0 = true;

    let events = BumpGoalCalculator::new().tag_goals(&[goal], &[team_bump, opponent_bump]);

    assert!(events.is_empty());
}

#[test]
fn bump_goal_rejects_stale_bumps() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let events = BumpGoalCalculator::new()
        .tag_goals(&[goal], &[bump_event(6.5, 65, player_id(2), player_id(3))]);

    assert!(events.is_empty());
}

#[test]
fn demo_goal_can_be_created_by_non_scorer_teammate() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let events = DemoGoalCalculator::new().tag_goals(&[goal], &[demo_event(9.1, 91, player_id(2))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::DemoGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Teammate));
    assert!(!has_modifier(&events[0], GoalTagModifier::ByScorer));
    assert_eq!(
        events[0]
            .tag
            .metadata()
            .evidence
            .iter()
            .find(|evidence| evidence.kind == GoalTagEvidenceKind::Demo)
            .and_then(|evidence| evidence.player.as_ref()),
        Some(&player_id(2))
    );
}

#[test]
fn demo_goal_marks_by_scorer_when_scorer_gets_demo() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let events = DemoGoalCalculator::new().tag_goals(&[goal], &[demo_event(9.1, 91, player_id(1))]);

    assert_eq!(tag_kinds(&events), vec![GoalTagKind::DemoGoal]);
    assert_eq!(performer(&events[0]), Some(GoalTagPerformer::Scorer));
    assert!(has_modifier(&events[0], GoalTagModifier::ByScorer));
}

#[test]
fn demo_goal_rejects_deaths_opponent_demos_and_stale_demos() {
    let goal = goal_with_touch(true, position(0.0, 2300.0, 120.0), Vec::new());
    let mut death = demo_event(9.1, 91, player_id(2));
    death.kind = TimelineEventKind::Death;
    let mut opponent_demo = demo_event(9.1, 91, player_id(3));
    opponent_demo.is_team_0 = Some(false);
    let stale_demo = demo_event(6.5, 65, player_id(2));

    let events = DemoGoalCalculator::new().tag_goals(&[goal], &[death, opponent_demo, stale_demo]);

    assert!(events.is_empty());
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
        .tag
        .metadata()
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

#[test]
fn goal_tag_definitions_are_unique_and_documented() {
    let mut ids = std::collections::BTreeSet::new();
    for definition in ALL_GOAL_TAG_DEFINITIONS {
        assert!(
            ids.insert(definition.id),
            "duplicate goal tag definition id {}",
            definition.id
        );
        assert!(
            !definition.summary.is_empty(),
            "{} should describe what the goal tag means",
            definition.id
        );
        assert!(
            !definition.approach.is_empty(),
            "{} should describe how the goal tag is assigned",
            definition.id
        );
    }
}
