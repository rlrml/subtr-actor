use super::*;

fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(player_id: PlayerId, is_team_0: bool, position: glam::Vec3, boost: f32) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
        boost_amount: Some(boost),
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn ball(y: f32) -> BallFrameState {
    ball_with_velocity(y, glam::Vec3::ZERO)
}

fn ball_with_velocity(y: f32, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, 92.0), velocity),
    })
}

fn touch(player: PlayerId, team_is_team_0: bool, frame: usize, time: f32) -> TouchEvent {
    TouchEvent {
        time,
        frame,
        team_is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

fn goal(scoring_team_is_team_0: bool, frame: usize, time: f32) -> GoalEvent {
    GoalEvent {
        time,
        frame,
        scoring_team_is_team_0,
        player: None,
        player_position: None,
        team_zero_score: None,
        team_one_score: None,
    }
}

fn speed_flip_event(player: PlayerId, is_team_0: bool, time: f32, frame: usize) -> SpeedFlipEvent {
    SpeedFlipEvent {
        time,
        frame,
        resolved_time: time + 0.25,
        resolved_frame: frame + 3,
        player,
        is_team_0,
        time_since_kickoff_start: time,
        start_position: [0.0, 0.0, 0.0],
        end_position: [0.0, 0.0, 0.0],
        start_speed: 900.0,
        max_speed: 1700.0,
        best_alignment: 0.95,
        initial_boost_alignment: 0.95,
        best_boost_alignment: 0.98,
        boost_alignment_sample_count: 4,
        dodge_delay_after_ground_leave_seconds: 0.0,
        diagonal_score: 0.95,
        estimated_dodge_impulse_magnitude: 300.0,
        estimated_dodge_impulse_forward_component: 0.6,
        estimated_dodge_impulse_side_component: 0.8,
        estimated_dodge_impulse_up_component: 0.1,
        cancel_score: 0.8,
        speed_score: 0.8,
        confidence: 0.9,
    }
}

#[test]
fn kickoff_goal_does_not_override_immediate_outcome() {
    let blue_taker = PlayerId::Steam(40);
    let orange_taker = PlayerId::Steam(41);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker, true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.2),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState {
                goal_events: vec![goal(false, 12, 1.2)],
                ..FrameEventsState::default()
            },
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.outcome, KickoffOutcome::TeamZeroWin);
    assert_eq!(event.winning_team_is_team_0, Some(true));
    assert!(event.kickoff_goal);
    assert_eq!(event.scoring_team_is_team_0, Some(false));
    assert_eq!(
        event.kickoff_possession_outcome,
        KickoffPossessionOutcome::TeamOnePossession
    );
    assert_eq!(event.kickoff_possession_team_is_team_0, Some(false));
}

#[test]
fn kickoff_records_movement_start_after_countdown() {
    let blue_taker = PlayerId::Steam(42);
    let orange_taker = PlayerId::Steam(43);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                game_state: Some(GAME_STATE_KICKOFF_COUNTDOWN),
                ball_has_been_hit: Some(false),
                kickoff_countdown_time: Some(3),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(-2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(30, 3.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                kickoff_countdown_time: Some(0),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(40, 4.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(320.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker, true, 40, 4.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(55, 5.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(420.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.start_time, 0.0);
    assert_eq!(event.start_frame, 0);
    assert_eq!(event.live_action_start_time, Some(3.0));
    assert_eq!(event.live_action_start_frame, Some(30));
    assert_eq!(event.movement_start_time, 3.0);
    assert_eq!(event.movement_start_frame, 30);
}

#[test]
fn kickoff_classifies_fake_and_missed_expected_takers() {
    let blue_taker = PlayerId::Steam(1);
    let blue_support = PlayerId::Steam(2);
    let orange_taker = PlayerId::Steam(3);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        blue_support.clone(),
                        true,
                        glam::Vec3::new(0.0, -4608.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_support.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState {
                players: vec![
                    player(blue_taker.clone(), true, glam::Vec3::ZERO, 21.0),
                    player(blue_support.clone(), true, glam::Vec3::ZERO, 12.0),
                    player(orange_taker.clone(), false, glam::Vec3::ZERO, 28.0),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.outcome, KickoffOutcome::TeamZeroWin);
    assert_eq!(event.win_strength, Some(2.0));
    assert_eq!(event.win_strength_band, KickoffWinStrengthBand::Clear);
    assert_eq!(
        event.team_zero_taker.as_ref().map(|player| &player.player),
        Some(&blue_taker)
    );
    assert_eq!(
        event.team_one_taker.as_ref().map(|player| &player.player),
        Some(&orange_taker)
    );
    assert_eq!(event.team_zero_non_takers.len(), 1);
    assert_eq!(&event.team_zero_non_takers[0].player, &blue_support);
    assert!(event.team_one_non_takers.is_empty());

    let blue_taker_event = event.team_zero_taker.as_ref().unwrap();
    assert_eq!(blue_taker_event.player, blue_taker);
    assert_eq!(blue_taker_event.outcome, KickoffTakerOutcome::Fake);
    assert_eq!(blue_taker_event.boost_after, Some(21.0));

    let blue_support_event = &event.team_zero_non_takers[0];
    assert_eq!(blue_support_event.player, blue_support);
    assert_eq!(blue_support_event.first_touch_time, Some(1.0));
    assert_eq!(blue_support_event.boost_after, Some(12.0));

    let orange_taker_event = event.team_one_taker.as_ref().unwrap();
    assert_eq!(orange_taker_event.player, orange_taker);
    assert_eq!(orange_taker_event.outcome, KickoffTakerOutcome::Missed);
}

#[test]
fn kickoff_classifies_support_players_as_cheating_or_going_for_boost() {
    let blue_taker = PlayerId::Steam(50);
    let blue_cheat = PlayerId::Steam(51);
    let orange_taker = PlayerId::Steam(52);
    let orange_boost = PlayerId::Steam(53);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        blue_cheat.clone(),
                        true,
                        glam::Vec3::new(0.0, -4608.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_boost.clone(),
                        false,
                        glam::Vec3::new(0.0, 4608.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(5, 0.5),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_cheat.clone(),
                        true,
                        glam::Vec3::new(0.0, -3900.0, 17.0),
                        31.0,
                    ),
                    player(
                        orange_boost.clone(),
                        false,
                        glam::Vec3::new(900.0, 4300.0, 17.0),
                        100.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState {
                players: vec![
                    player(blue_taker.clone(), true, glam::Vec3::ZERO, 20.0),
                    player(
                        blue_cheat.clone(),
                        true,
                        glam::Vec3::new(0.0, -3600.0, 17.0),
                        30.0,
                    ),
                    player(orange_taker.clone(), false, glam::Vec3::ZERO, 25.0),
                    player(
                        orange_boost.clone(),
                        false,
                        glam::Vec3::new(1200.0, 4300.0, 17.0),
                        100.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    let blue_taker_event = event.team_zero_taker.as_ref().unwrap();
    assert_eq!(blue_taker_event.player, blue_taker);

    let blue_cheat_event = event
        .team_zero_non_takers
        .iter()
        .find(|player| player.player == blue_cheat)
        .unwrap();
    assert_eq!(
        blue_cheat_event.support_behavior,
        KickoffSupportBehavior::Cheat
    );

    let orange_boost_event = event
        .team_one_non_takers
        .iter()
        .find(|player| player.player == orange_boost)
        .unwrap();
    assert_eq!(
        orange_boost_event.support_behavior,
        KickoffSupportBehavior::GoForBoost
    );

    let mut stats = KickoffStatsAccumulator::new();
    stats.apply_event(event);
    assert_eq!(
        stats
            .player_stats()
            .get(&blue_cheat)
            .unwrap()
            .support_cheats,
        1
    );
    assert_eq!(
        stats
            .player_stats()
            .get(&orange_boost)
            .unwrap()
            .support_go_for_boosts,
        1
    );
}

#[test]
fn kickoff_tracks_first_touch_taker_delay_exit_velocity_and_follow_up() {
    let blue_taker = PlayerId::Steam(20);
    let orange_taker = PlayerId::Steam(21);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.15),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 12, 1.15)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(14, 1.4),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 14, 1.4)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball_with_velocity(360.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.first_touch_player, Some(blue_taker.clone()));
    assert_eq!(event.first_touch_team_is_team_0, Some(true));
    assert_eq!(event.team_zero_taker_touch_time, Some(1.0));
    assert_eq!(event.team_zero_taker_touch_frame, Some(10));
    assert_eq!(event.team_one_taker_touch_time, Some(1.15));
    assert_eq!(event.team_one_taker_touch_frame, Some(12));
    assert!((event.taker_touch_delay_seconds.unwrap() - 0.15).abs() < 0.0001);
    assert_eq!(event.exit_velocity, Some([0.0, 500.0, 0.0]));
    assert_eq!(event.exit_speed, Some(500.0));
    assert_eq!(event.exit_y_velocity, Some(500.0));
    assert_eq!(event.first_follow_up_touch_time, None);
    assert_eq!(event.first_follow_up_touch_frame, None);
    assert_eq!(event.first_follow_up_touch_team_is_team_0, None);
    assert_eq!(event.first_follow_up_touch_player, None);
    assert_eq!(
        event.kickoff_possession_outcome,
        KickoffPossessionOutcome::Contested
    );
    assert_eq!(event.kickoff_possession_team_is_team_0, None);
}

#[test]
fn kickoff_taker_touch_delay_is_non_negative_when_team_one_touches_first() {
    let blue_taker = PlayerId::Steam(26);
    let orange_taker = PlayerId::Steam(27);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.2),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 12, 1.2)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(-360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.team_zero_taker_touch_time, Some(1.2));
    assert_eq!(event.team_one_taker_touch_time, Some(1.0));
    assert!((event.taker_touch_delay_seconds.unwrap() - 0.2).abs() < 0.0001);
}

#[test]
fn kickoff_follow_up_clean_possession_uses_unchallenged_touch_sequence() {
    let blue_taker = PlayerId::Steam(22);
    let orange_taker = PlayerId::Steam(23);
    let orange_support = PlayerId::Steam(230);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_support.clone(),
                        false,
                        glam::Vec3::new(0.0, 4608.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.15),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 12, 1.15)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(13, 1.2),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_support.clone(), false, 13, 1.2)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(14, 1.35),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 14, 1.35)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(18, 1.8),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 18, 1.8)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.outcome, KickoffOutcome::TeamZeroWin);
    assert_eq!(event.first_follow_up_touch_time, Some(1.2));
    assert_eq!(event.first_follow_up_touch_frame, Some(13));
    assert_eq!(event.first_follow_up_touch_team_is_team_0, Some(false));
    assert_eq!(event.first_follow_up_touch_player, Some(orange_support));
    assert_eq!(
        event.kickoff_possession_outcome,
        KickoffPossessionOutcome::TeamOnePossession
    );
    assert_eq!(event.kickoff_possession_team_is_team_0, Some(false));
}

#[test]
fn kickoff_goal_preserves_actual_follow_up_contest() {
    let blue_taker = PlayerId::Steam(220);
    let orange_taker = PlayerId::Steam(221);
    let orange_support = PlayerId::Steam(222);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_support.clone(),
                        false,
                        glam::Vec3::new(0.0, 4608.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.15),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 12, 1.15)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(13, 1.2),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_support.clone(), false, 13, 1.2)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(15, 1.4),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 15, 1.4)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(16, 1.6),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState {
                goal_events: vec![goal(false, 16, 1.6)],
                ..FrameEventsState::default()
            },
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert!(event.kickoff_goal);
    assert_eq!(event.scoring_team_is_team_0, Some(false));
    assert_eq!(event.first_follow_up_touch_time, Some(1.2));
    assert_eq!(event.first_follow_up_touch_team_is_team_0, Some(false));
    assert_eq!(
        event.kickoff_possession_outcome,
        KickoffPossessionOutcome::Contested
    );
    assert_eq!(event.kickoff_possession_team_is_team_0, None);
}

#[test]
fn kickoff_possession_outcome_tracks_team_advantage_before_late_challenge() {
    let blue_taker = PlayerId::Steam(28);
    let orange_taker = PlayerId::Steam(29);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(12, 1.15),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 12, 1.15)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(18, 1.6),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(orange_taker.clone(), false, 18, 1.6)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(21, 2.05),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 21, 2.05)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(event.first_follow_up_touch_time, Some(1.6));
    assert_eq!(event.first_follow_up_touch_frame, Some(18));
    assert_eq!(event.first_follow_up_touch_team_is_team_0, Some(false));
    assert_eq!(event.first_follow_up_touch_player, Some(orange_taker));
    assert_eq!(
        event.kickoff_possession_outcome,
        KickoffPossessionOutcome::TeamOneAdvantage
    );
    assert_eq!(event.kickoff_possession_team_is_team_0, Some(false));
}

#[test]
fn kickoff_uses_speed_flip_events_as_approach_source_of_truth() {
    let blue_taker = PlayerId::Steam(24);
    let orange_taker = PlayerId::Steam(25);
    let speed_flip = speed_flip_event(blue_taker.clone(), true, 0.4, 4);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update_with_speed_flips(KickoffUpdateContext {
            frame: &frame(0, 0.0),
            gameplay: &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            ball: &ball(0.0),
            players: &PlayerFrameState {
                players: vec![
                    player(
                        blue_taker.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_taker.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            touch_state: &TouchState::default(),
            events: &FrameEventsState::default(),
            speed_flip_events: std::slice::from_ref(&speed_flip),
        })
        .unwrap();

    calculator
        .update_with_speed_flips(KickoffUpdateContext {
            frame: &frame(10, 1.0),
            gameplay: &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            ball: &ball(0.0),
            players: &PlayerFrameState::default(),
            touch_state: &TouchState {
                touch_events: vec![touch(blue_taker.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            events: &FrameEventsState::default(),
            speed_flip_events: std::slice::from_ref(&speed_flip),
        })
        .unwrap();

    calculator
        .update_with_speed_flips(KickoffUpdateContext {
            frame: &frame(25, 2.5),
            gameplay: &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            ball: &ball(360.0),
            players: &PlayerFrameState::default(),
            touch_state: &TouchState::default(),
            events: &FrameEventsState::default(),
            speed_flip_events: std::slice::from_ref(&speed_flip),
        })
        .unwrap();

    let event = calculator.events().last().unwrap();
    let blue_taker_event = event.team_zero_taker.as_ref().unwrap();
    assert_eq!(blue_taker_event.player, blue_taker);
    assert_eq!(blue_taker_event.approach, KickoffApproach::SpeedFlip);
}

#[test]
fn kickoff_classifies_known_taker_approaches() {
    let speed_flip = KickoffPlayerSnapshot {
        player: PlayerId::Steam(30),
        is_team_0: true,
        start_position: [-2048.0, -2560.0, 17.0],
        spawn_position: KickoffSpawnPosition::DiagonalLeft,
        start_boost: Some(33.0),
        first_touch_time: Some(1.0),
        first_touch_frame: Some(10),
        approach_trace: KickoffApproachTrace {
            boost_active_sample_count: 3,
            first_dodge_time: Some(0.35),
            first_dodge_frame: Some(4),
            first_dodge_forward_component: Some(0.55),
            first_dodge_side_component: Some(0.75),
            max_speed: 1700.0,
            min_boost: Some(20.0),
            ..KickoffApproachTrace::default()
        },
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &speed_flip,
            KickoffTakerOutcome::Touched,
            Some(20.0),
            true,
        ),
        KickoffApproach::SpeedFlip
    );

    assert_eq!(
        KickoffCalculator::classify_approach(
            &speed_flip,
            KickoffTakerOutcome::Touched,
            Some(20.0),
            false,
        ),
        KickoffApproach::DiagonalFlip
    );

    let front_flip = KickoffPlayerSnapshot {
        approach_trace: KickoffApproachTrace {
            first_dodge_time: Some(0.35),
            first_dodge_frame: Some(4),
            first_dodge_forward_component: Some(0.8),
            first_dodge_side_component: Some(0.1),
            max_speed: 1200.0,
            ..KickoffApproachTrace::default()
        },
        ..speed_flip.clone()
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &front_flip,
            KickoffTakerOutcome::Touched,
            Some(28.0),
            false,
        ),
        KickoffApproach::FrontFlip
    );

    let boost_into_ball = KickoffPlayerSnapshot {
        first_touch_time: Some(0.95),
        first_touch_frame: Some(9),
        approach_trace: KickoffApproachTrace {
            boost_active_sample_count: 4,
            min_boost: Some(18.0),
            max_speed: 1100.0,
            ..KickoffApproachTrace::default()
        },
        ..speed_flip.clone()
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &boost_into_ball,
            KickoffTakerOutcome::Touched,
            Some(18.0),
            false,
        ),
        KickoffApproach::BoostIntoBall
    );

    let missed_diagonal_flip = KickoffPlayerSnapshot {
        first_touch_time: None,
        first_touch_frame: None,
        approach_trace: KickoffApproachTrace {
            boost_active_sample_count: 3,
            first_dodge_time: Some(0.35),
            first_dodge_frame: Some(4),
            first_dodge_forward_component: Some(0.55),
            first_dodge_side_component: Some(0.75),
            min_boost: Some(20.0),
            last_position: Some([-800.0, -1000.0, 17.0]),
            ..KickoffApproachTrace::default()
        },
        ..speed_flip.clone()
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &missed_diagonal_flip,
            KickoffTakerOutcome::Missed,
            Some(20.0),
            false,
        ),
        KickoffApproach::DiagonalFlip
    );

    let missed_boost_into_ball = KickoffPlayerSnapshot {
        first_touch_time: None,
        first_touch_frame: None,
        approach_trace: KickoffApproachTrace {
            boost_active_sample_count: 4,
            min_boost: Some(20.0),
            last_position: Some([-800.0, -1000.0, 17.0]),
            ..KickoffApproachTrace::default()
        },
        ..speed_flip.clone()
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &missed_boost_into_ball,
            KickoffTakerOutcome::Missed,
            Some(116.0),
            false,
        ),
        KickoffApproach::BoostIntoBall
    );

    let fake_go_for_boost = KickoffPlayerSnapshot {
        first_touch_time: None,
        first_touch_frame: None,
        approach_trace: KickoffApproachTrace {
            boost_active_sample_count: 1,
            min_boost: Some(26.0),
            last_position: Some([-2560.0, -3584.0, 17.0]),
            ..KickoffApproachTrace::default()
        },
        ..speed_flip
    };
    assert_eq!(
        KickoffCalculator::classify_approach(
            &fake_go_for_boost,
            KickoffTakerOutcome::Fake,
            Some(26.0),
            false,
        ),
        KickoffApproach::FakeGoForBoost
    );
}

#[test]
fn kickoff_tie_breaks_expected_taker_by_actual_touch_then_left_goes() {
    let left_player = PlayerId::Steam(10);
    let right_player = PlayerId::Steam(11);
    let orange_player = PlayerId::Steam(12);
    let mut calculator = KickoffCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &GameplayState {
                ball_has_been_hit: Some(false),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState {
                players: vec![
                    player(
                        left_player.clone(),
                        true,
                        glam::Vec3::new(-2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        right_player.clone(),
                        true,
                        glam::Vec3::new(2048.0, -2560.0, 17.0),
                        33.0,
                    ),
                    player(
                        orange_player.clone(),
                        false,
                        glam::Vec3::new(2048.0, 2560.0, 17.0),
                        33.0,
                    ),
                ],
            },
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(0.0),
            &PlayerFrameState::default(),
            &TouchState {
                touch_events: vec![touch(right_player.clone(), true, 10, 1.0)],
                ..TouchState::default()
            },
            &FrameEventsState::default(),
        )
        .unwrap();

    calculator
        .update(
            &frame(25, 2.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &ball(360.0),
            &PlayerFrameState::default(),
            &TouchState::default(),
            &FrameEventsState::default(),
        )
        .unwrap();

    let event = calculator.events().last().unwrap();
    assert_eq!(
        event.team_zero_taker.as_ref().map(|player| &player.player),
        Some(&right_player)
    );
    assert!(event
        .team_zero_non_takers
        .iter()
        .any(|player| player.player == left_player));

    let left_goes_index = KickoffCalculator::expected_taker_by_team(
        &[
            KickoffPlayerSnapshot {
                player: left_player,
                is_team_0: true,
                start_position: [-2048.0, -2560.0, 17.0],
                spawn_position: KickoffSpawnPosition::DiagonalLeft,
                start_boost: Some(33.0),
                first_touch_time: None,
                first_touch_frame: None,
                approach_trace: KickoffApproachTrace::default(),
            },
            KickoffPlayerSnapshot {
                player: right_player,
                is_team_0: true,
                start_position: [2048.0, -2560.0, 17.0],
                spawn_position: KickoffSpawnPosition::DiagonalRight,
                start_boost: Some(33.0),
                first_touch_time: None,
                first_touch_frame: None,
                approach_trace: KickoffApproachTrace::default(),
            },
        ],
        true,
    );
    assert_eq!(left_goes_index, Some(0));
}

#[test]
fn kickoff_taker_prefers_ball_committer_when_no_touch() {
    // Regression for a real 2v2 kickoff (a contested 50/50). The orange
    // challenger drove in and contacted the ball, but the replay only emitted
    // the opposing team's BallHitTeamNum marker, so subtr-actor recorded no
    // touch for either orange player. Both orange players spawn on mirrored
    // diagonals at equal distance from the ball, so the distance tie and the
    // first-touch tiebreak can't disambiguate. The legacy fallback then picked
    // the geometrically leftmost player (index 0) -- a back/cheat player that
    // stayed ~3400uu from the ball -- instead of the challenger who drove from
    // spawn into the ball and burned all boost (index 1). Selection must follow
    // the commitment signal (advance toward the ball, then boost burned), not
    // the static left-side tiebreak.
    let kept_boost_player = PlayerId::Epic("kept-boost".to_owned());
    let ball_committer = PlayerId::Epic("ball-committer".to_owned());

    let players = [
        KickoffPlayerSnapshot {
            player: kept_boost_player,
            is_team_0: false,
            start_position: [2048.0, 2560.0, 17.0],
            spawn_position: KickoffSpawnPosition::DiagonalLeft,
            start_boost: Some(85.0),
            first_touch_time: None,
            first_touch_frame: None,
            approach_trace: KickoffApproachTrace {
                min_boost: Some(85.0),
                last_position: Some([1900.0, 2400.0, 17.0]),
                ..KickoffApproachTrace::default()
            },
        },
        KickoffPlayerSnapshot {
            player: ball_committer,
            is_team_0: false,
            start_position: [-2048.0, 2560.0, 17.0],
            spawn_position: KickoffSpawnPosition::DiagonalRight,
            start_boost: Some(85.0),
            first_touch_time: None,
            first_touch_frame: None,
            approach_trace: KickoffApproachTrace {
                min_boost: Some(0.0),
                last_position: Some([-300.0, 700.0, 17.0]),
                ..KickoffApproachTrace::default()
            },
        },
    ];

    // Legacy `relative_left_value` tiebreak would have returned Some(0).
    assert_eq!(
        KickoffCalculator::expected_taker_by_team(&players, false),
        Some(1)
    );
}

#[test]
fn kickoff_stats_accumulate_boost_strength_fake_and_miss_counts() {
    let player_id = PlayerId::Steam(1);
    let event = KickoffEvent {
        start_time: 0.0,
        start_frame: 0,
        end_time: 1.5,
        end_frame: 15,
        live_action_start_time: Some(0.0),
        live_action_start_frame: Some(0),
        movement_start_time: 0.0,
        movement_start_frame: 0,
        kickoff_type: KickoffType::Center,
        kickoff_direction: KickoffDirection::Center,
        first_touch_time: Some(0.5),
        first_touch_frame: Some(5),
        first_touch_team_is_team_0: Some(true),
        first_touch_player: Some(player_id.clone()),
        team_zero_taker_touch_time: None,
        team_zero_taker_touch_frame: None,
        team_one_taker_touch_time: None,
        team_one_taker_touch_frame: None,
        taker_touch_delay_seconds: None,
        exit_velocity: Some([0.0, 300.0, 0.0]),
        exit_speed: Some(300.0),
        exit_y_velocity: Some(300.0),
        first_follow_up_touch_time: Some(0.75),
        first_follow_up_touch_frame: Some(8),
        first_follow_up_touch_team_is_team_0: Some(true),
        first_follow_up_touch_player: Some(player_id.clone()),
        outcome: KickoffOutcome::TeamZeroWin,
        winning_team_is_team_0: Some(true),
        win_strength: Some(1.5),
        win_strength_band: KickoffWinStrengthBand::Clear,
        kickoff_possession_outcome: KickoffPossessionOutcome::TeamZeroPossession,
        kickoff_possession_team_is_team_0: Some(true),
        kickoff_goal: true,
        scoring_team_is_team_0: Some(true),
        time_to_goal: Some(4.0),
        team_zero_taker: Some(KickoffTakerEvent {
            player: player_id.clone(),
            is_team_0: true,
            start_position: [0.0, -4608.0, 17.0],
            spawn_position: KickoffSpawnPosition::Center,
            start_boost: Some(33.0),
            boost_after: Some(11.0),
            first_touch_time: None,
            first_touch_frame: None,
            outcome: KickoffTakerOutcome::Fake,
            approach: KickoffApproach::FakeGoForBoost,
        }),
        team_one_taker: None,
        team_zero_non_takers: Vec::new(),
        team_one_non_takers: Vec::new(),
    };
    let mut stats = KickoffStatsAccumulator::new();

    stats.apply_event(&event);

    assert_eq!(stats.stats().count, 1);
    assert_eq!(stats.stats().team_zero_wins, 1);
    assert_eq!(stats.stats().kickoff_goal_count, 1);
    assert_eq!(stats.stats().team_zero_kickoff_goals, 1);
    assert_eq!(stats.stats().team_one_kickoff_goals, 0);
    assert_eq!(stats.stats().team_zero_kickoff_possession_advantages, 0);
    assert_eq!(stats.stats().team_one_kickoff_possession_advantages, 0);
    assert_eq!(stats.stats().fake_count, 1);
    assert_eq!(stats.stats().missed_count, 0);
    assert_eq!(stats.stats().average_win_strength(), 1.5);
    assert_eq!(stats.stats().average_boost_after(), 11.0);
    let team_zero_stats = stats.stats().for_team(true);
    assert_eq!(team_zero_stats.kickoff_possession_advantages, 0);
    assert_eq!(team_zero_stats.opponent_kickoff_possession_advantages, 0);
    assert_eq!(team_zero_stats.kickoff_goals_for, 1);
    assert_eq!(team_zero_stats.kickoff_goals_against, 0);
    let team_one_stats = stats.stats().for_team(false);
    assert_eq!(team_one_stats.kickoff_goals_for, 0);
    assert_eq!(team_one_stats.kickoff_goals_against, 1);
    let player_stats = stats.player_stats().get(&player_id).unwrap();
    assert_eq!(player_stats.fakes, 1);
    assert_eq!(player_stats.support_cheats, 0);
    assert_eq!(player_stats.support_go_for_boosts, 0);
    assert_eq!(player_stats.kickoff_goal_count, 1);
    assert_eq!(player_stats.average_boost_after(), 11.0);
}

#[test]
fn kickoff_type_only_names_symmetric_taker_spawns() {
    assert_eq!(
        KickoffType::from_taker_spawns(
            Some(KickoffSpawnPosition::DiagonalLeft),
            Some(KickoffSpawnPosition::DiagonalLeft),
        ),
        KickoffType::Diagonal
    );
    assert_eq!(
        KickoffType::from_taker_spawns(
            Some(KickoffSpawnPosition::OffCenterRight),
            Some(KickoffSpawnPosition::OffCenterRight),
        ),
        KickoffType::CenterOffset
    );
    assert_eq!(
        KickoffType::from_taker_spawns(
            Some(KickoffSpawnPosition::OffCenterLeft),
            Some(KickoffSpawnPosition::OffCenterLeft),
        ),
        KickoffType::CenterOffset
    );
    assert_eq!(
        KickoffType::from_taker_spawns(
            Some(KickoffSpawnPosition::Center),
            Some(KickoffSpawnPosition::Center),
        ),
        KickoffType::Center
    );
    assert_eq!(
        KickoffType::from_taker_spawns(
            Some(KickoffSpawnPosition::DiagonalLeft),
            Some(KickoffSpawnPosition::DiagonalRight),
        ),
        KickoffType::Unknown
    );
}

#[test]
fn kickoff_direction_tracks_symmetric_taker_spawn_side() {
    assert_eq!(
        KickoffDirection::from_taker_spawns(
            Some(KickoffSpawnPosition::DiagonalLeft),
            Some(KickoffSpawnPosition::DiagonalLeft),
        ),
        KickoffDirection::Left
    );
    assert_eq!(
        KickoffDirection::from_taker_spawns(
            Some(KickoffSpawnPosition::OffCenterRight),
            Some(KickoffSpawnPosition::OffCenterRight),
        ),
        KickoffDirection::Right
    );
    assert_eq!(
        KickoffDirection::from_taker_spawns(
            Some(KickoffSpawnPosition::Center),
            Some(KickoffSpawnPosition::Center),
        ),
        KickoffDirection::Center
    );
    assert_eq!(
        KickoffDirection::from_taker_spawns(
            Some(KickoffSpawnPosition::DiagonalLeft),
            Some(KickoffSpawnPosition::DiagonalRight),
        ),
        KickoffDirection::Unknown
    );
}
