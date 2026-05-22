use super::*;

fn test_player(
    player_id: PlayerId,
    boost_amount: f32,
    last_boost_amount: f32,
    position: glam::Vec3,
) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0: true,
        rigid_body: Some(boxcars::RigidBody {
            sleeping: false,
            location: glam_to_vec(&position),
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: None,
            angular_velocity: None,
        }),
        boost_amount: Some(boost_amount),
        last_boost_amount: Some(last_boost_amount),
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

#[test]
fn records_inactive_pickup_without_active_collection() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Small)
        .copied()
        .expect("standard layout should include small pads");
    let player = test_player(
        player_id.clone(),
        BOOST_KICKOFF_START_AMOUNT + SMALL_PAD_AMOUNT_RAW,
        0.0,
        pad_position,
    );

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &GameplayState {
                game_state: Some(GAME_STATE_GOAL_SCORED_REPLAY),
                ball_has_been_hit: Some(true),
                ..GameplayState::default()
            },
            &PlayerFrameState {
                players: vec![player],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.0,
                    frame: 1,
                    pad_id: "inactive-small-pad".to_string(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            false,
        )
        .expect("inactive boost update should succeed");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.amount_collected, 0.0);
    assert_eq!(player_stats.small_pads_collected, 0);
    assert_eq!(player_stats.amount_collected_inactive, SMALL_PAD_AMOUNT_RAW);
    assert_eq!(player_stats.small_pads_collected_inactive, 1);
    assert_eq!(calculator.team_zero_stats().amount_collected, 0.0);
    assert_eq!(
        calculator.team_zero_stats().amount_collected_inactive,
        SMALL_PAD_AMOUNT_RAW
    );
    assert_eq!(
        calculator.team_zero_stats().small_pads_collected_inactive,
        1
    );
}

#[test]
fn counts_reused_pickup_sequence_after_pad_respawn() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Big)
        .copied()
        .expect("standard layout should include big pads");
    let pad_id = "reused-sequence-big-pad".to_string();
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(player_id.clone(), 100.0, 0.0, pad_position)],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.0,
                    frame: 1,
                    pad_id: pad_id.clone(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            true,
        )
        .expect("first boost update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 2,
                time: 11.1,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(player_id.clone(), 100.0, 100.0, pad_position)],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 11.1,
                    frame: 2,
                    pad_id: pad_id.clone(),
                    player: None,
                    kind: BoostPadEventKind::Available,
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            true,
        )
        .expect("pad availability update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 3,
                time: 11.2,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(player_id.clone(), 200.0, 100.0, pad_position)],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 11.2,
                    frame: 3,
                    pad_id,
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            true,
        )
        .expect("second boost update should succeed");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.big_pads_collected, 2);
    assert_eq!(calculator.team_zero_stats().big_pads_collected, 2);
}

#[test]
fn counts_pickup_after_respawn_without_available_event() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Big)
        .copied()
        .expect("standard layout should include big pads");
    let pad_id = "missing-available-big-pad".to_string();
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    for (frame_number, time, sequence, previous_boost, boost_amount) in
        [(1, 1.0, 7, 0.0, 100.0), (2, 11.2, 9, 100.0, 200.0)]
    {
        calculator
            .update_parts(
                &FrameInfo {
                    frame_number,
                    time,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        boost_amount,
                        previous_boost,
                        pad_position,
                    )],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time,
                        frame: frame_number,
                        pad_id: pad_id.clone(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("boost update should succeed");
    }

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.big_pads_collected, 2);
    assert_eq!(calculator.team_zero_stats().big_pads_collected, 2);
}

#[test]
fn skips_inactive_pickup_without_observed_boost_gain() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Big)
        .copied()
        .expect("standard layout should include big pads");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &GameplayState::default(),
            &PlayerFrameState {
                players: vec![test_player(player_id.clone(), 100.0, 100.0, pad_position)],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.0,
                    frame: 1,
                    pad_id: "inactive-no-gain-big-pad".to_string(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            false,
        )
        .expect("boost update should succeed");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.big_pads_collected_inactive, 0);
    assert_eq!(player_stats.amount_collected_inactive, 0.0);
}

#[test]
fn observed_boost_increases_do_not_emit_pickup_events_without_reported_pad_pickups() {
    let mut calculator = BoostCalculator::new();
    let small_player = PlayerId::Steam(1);
    let big_player = PlayerId::Steam(2);
    let ambiguous_player = PlayerId::Steam(3);
    let respawn_player = PlayerId::Steam(4);
    let two_small_player = PlayerId::Steam(5);
    let position = glam::Vec3::ZERO;
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![
                    test_player(small_player.clone(), 10.0, 10.0, position),
                    test_player(big_player.clone(), 10.0, 10.0, position),
                    test_player(ambiguous_player.clone(), 230.0, 230.0, position),
                    test_player(respawn_player.clone(), 0.0, 0.0, position),
                    test_player(
                        two_small_player.clone(),
                        BOOST_KICKOFF_START_AMOUNT,
                        BOOST_KICKOFF_START_AMOUNT,
                        position,
                    ),
                ],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            true,
        )
        .expect("first boost update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 2,
                time: 1.1,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![
                    test_player(
                        small_player.clone(),
                        10.0 + SMALL_PAD_AMOUNT_RAW,
                        10.0,
                        position,
                    ),
                    test_player(big_player.clone(), BOOST_MAX_AMOUNT, 10.0, position),
                    test_player(ambiguous_player.clone(), BOOST_MAX_AMOUNT, 230.0, position),
                    test_player(
                        respawn_player.clone(),
                        BOOST_KICKOFF_START_AMOUNT,
                        0.0,
                        position,
                    ),
                    test_player(
                        two_small_player.clone(),
                        BOOST_KICKOFF_START_AMOUNT + 2.0 * SMALL_PAD_AMOUNT_RAW,
                        BOOST_KICKOFF_START_AMOUNT,
                        position,
                    ),
                ],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            true,
        )
        .expect("second boost update should succeed");

    calculator
        .finish_calculation()
        .expect("pending inferred pickups should be discarded");
    let events = calculator.pickup_comparison_events();
    assert!(events.is_empty());
    assert!(calculator.player_stats().get(&respawn_player).is_some());
}

#[test]
fn reported_pickup_without_observed_boost_increase_is_emitted_as_counted_pickup() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Small)
        .copied()
        .expect("standard layout should include small pads");
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(
                    player_id.clone(),
                    BOOST_MAX_AMOUNT,
                    BOOST_MAX_AMOUNT,
                    pad_position,
                )],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            true,
        )
        .expect("first boost update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 2,
                time: 1.1,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(
                    player_id.clone(),
                    BOOST_MAX_AMOUNT,
                    BOOST_MAX_AMOUNT,
                    pad_position,
                )],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.1,
                    frame: 2,
                    pad_id: "full-boost-small-pad".to_string(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            true,
        )
        .expect("second boost update should succeed");

    calculator
        .finish_calculation()
        .expect("pickup comparisons should finish");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should exist");
    assert_eq!(player_stats.small_pads_collected, 1);
    assert_eq!(player_stats.amount_collected_small, 0.0);
    assert_eq!(player_stats.overfill_total, SMALL_PAD_AMOUNT_RAW);

    let events = calculator.pickup_comparison_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].comparison, BoostPickupComparison::Both);
    assert_eq!(events[0].pad_type, BoostPickupPadType::Small);
    assert_eq!(events[0].reported_frame, Some(2));
    assert_eq!(events[0].inferred_frame, None);
}

#[test]
fn matches_two_small_pickups_from_one_observed_boost_increase() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Small)
        .copied()
        .expect("standard layout should include small pads");
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 1,
                time: 1.0,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(
                    player_id.clone(),
                    BOOST_KICKOFF_START_AMOUNT,
                    BOOST_KICKOFF_START_AMOUNT,
                    pad_position,
                )],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            true,
        )
        .expect("first boost update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 2,
                time: 1.1,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(
                    player_id.clone(),
                    BOOST_KICKOFF_START_AMOUNT + 2.0 * SMALL_PAD_AMOUNT_RAW,
                    BOOST_KICKOFF_START_AMOUNT,
                    pad_position,
                )],
            },
            &FrameEventsState {
                boost_pad_events: vec![
                    BoostPadEvent {
                        time: 1.1,
                        frame: 2,
                        pad_id: "small-pad-one".to_string(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    },
                    BoostPadEvent {
                        time: 1.1,
                        frame: 2,
                        pad_id: "small-pad-two".to_string(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    },
                ],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            true,
        )
        .expect("second boost update should succeed");

    calculator
        .finish_calculation()
        .expect("pickup comparisons should flush");

    let events = calculator.pickup_comparison_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| {
        event.comparison == BoostPickupComparison::Both
            && event.pad_type == BoostPickupPadType::Small
    }));
}
