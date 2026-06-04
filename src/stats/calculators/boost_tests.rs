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
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::default(),
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
fn boost_ledger_replays_respawn_collection_and_use_totals() {
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
            &LivePlayState::active_play(),
        )
        .expect("initial boost update should succeed");

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
                players: vec![test_player(player_id.clone(), 20.0, 33.0, pad_position)],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
        )
        .expect("boost use update should succeed");

    calculator
        .update_parts(
            &FrameInfo {
                frame_number: 3,
                time: 1.2,
                dt: 1.0 / 30.0,
                seconds_remaining: None,
            },
            &active_gameplay,
            &PlayerFrameState {
                players: vec![test_player(
                    player_id.clone(),
                    20.0 + SMALL_PAD_AMOUNT_RAW,
                    20.0,
                    pad_position,
                )],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.2,
                    frame: 3,
                    pad_id: "ledger-small-pad".to_string(),
                    player: Some(player_id.clone()),
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
        )
        .expect("boost collection update should succeed");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    let projected_ledger_events = calculator.projected_ledger_events();
    let ledger_sum = |transaction| {
        projected_ledger_events
            .iter()
            .filter(|event| event.transaction == transaction && event.player_id == player_id)
            .map(|event| event.amount)
            .sum::<f32>()
    };

    assert!(
        (ledger_sum(BoostLedgerTransactionKind::Respawn) - player_stats.amount_respawned).abs()
            < 0.001
    );
    assert!(
        (ledger_sum(BoostLedgerTransactionKind::Collected) - player_stats.amount_collected).abs()
            < 0.001
    );
    assert!(
        (ledger_sum(BoostLedgerTransactionKind::Stolen) - player_stats.amount_stolen).abs() < 0.001
    );
    assert!(
        (ledger_sum(BoostLedgerTransactionKind::Used) - player_stats.amount_used).abs() < 0.001
    );

    let mut reconstructed = BoostStatsAccumulator::new();
    for event in calculator.projected_state_events() {
        reconstructed.apply_state_event(&event);
    }
    for event in calculator.projected_ledger_events() {
        reconstructed.apply_ledger_event(&event);
    }
    assert_eq!(reconstructed.player_stats(), calculator.player_stats());
    assert_eq!(
        reconstructed.team_zero_stats(),
        calculator.team_zero_stats()
    );
    assert_eq!(reconstructed.team_one_stats(), calculator.team_one_stats());
}

#[test]
fn reported_small_pickup_while_boosting_infers_same_sample_use() {
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
                players: vec![test_player(player_id.clone(), 20.0, 20.0, pad_position)],
            },
            &FrameEventsState::default(),
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
        )
        .expect("initial boost update should succeed");

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
                    20.0 + SMALL_PAD_AMOUNT_RAW - 4.0,
                    20.0,
                    pad_position,
                )],
            },
            &FrameEventsState {
                boost_pad_events: vec![BoostPadEvent {
                    time: 1.1,
                    frame: 2,
                    pad_id: "boosting-small-pad".to_string(),
                    player: Some(player_id.clone()),
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
        )
        .expect("small pad pickup update should succeed");

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert!(
        (player_stats.amount_collected_small - SMALL_PAD_AMOUNT_RAW).abs() < 0.001,
        "reported small pad should be fully credited even though net gain is smaller"
    );
    assert!(
        (player_stats.amount_used - (BOOST_KICKOFF_START_AMOUNT - 20.0 + 4.0)).abs() < 0.001,
        "same-sample boost use should be inferred from credited pickup and observed current boost"
    );
}

#[test]
fn demo_reset_does_not_count_removed_boost_as_used() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let attacker_id = PlayerId::Steam(2);
    let player_position = glam::Vec3::new(0.0, 0.0, 17.0);
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };
    let update = |calculator: &mut BoostCalculator,
                  frame_number,
                  time,
                  boost_amount,
                  last_boost_amount,
                  events: FrameEventsState| {
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
                        last_boost_amount,
                        player_position,
                    )],
                },
                &events,
                &PlayerVerticalState::default(),
                &LivePlayState::active_play(),
            )
            .expect("boost update should succeed");
    };

    update(
        &mut calculator,
        1,
        1.0,
        120.0,
        120.0,
        FrameEventsState::default(),
    );
    update(
        &mut calculator,
        2,
        1.1,
        0.0,
        120.0,
        FrameEventsState {
            demo_events: vec![DemolishInfo {
                time: 1.1,
                seconds_remaining: 299,
                frame: 2,
                attacker: attacker_id,
                victim: player_id.clone(),
                attacker_velocity: glam_to_vec(&glam::Vec3::ZERO),
                victim_velocity: glam_to_vec(&glam::Vec3::ZERO),
                attacker_location: None,
                victim_location: glam_to_vec(&player_position),
            }],
            ..FrameEventsState::default()
        },
    );
    update(
        &mut calculator,
        3,
        2.0,
        0.0,
        0.0,
        FrameEventsState::default(),
    );
    update(
        &mut calculator,
        4,
        4.4,
        BOOST_KICKOFF_START_AMOUNT,
        0.0,
        FrameEventsState::default(),
    );

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.amount_used, 0.0);
    assert_eq!(player_stats.amount_used_while_grounded, 0.0);
    assert_eq!(calculator.team_zero_stats().amount_used, 0.0);
    assert!(calculator
        .ledger_events()
        .iter()
        .filter(|event| event.player_id == player_id)
        .all(
            |event| event.transaction != BoostLedgerTransactionKind::Used
                && event.transaction != BoostLedgerTransactionKind::UsedAllocation
        ));
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
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
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
                    player_position: None,
                    kind: BoostPadEventKind::Available,
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
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
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
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
                        player_position: None,
                        kind: BoostPadEventKind::PickedUp { sequence },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                &LivePlayState::active_play(),
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
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 7 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::default(),
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
fn stale_unreported_boost_increase_is_counted_as_ghost_pickup() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Big)
        .copied()
        .expect("standard layout should include big pads");
    let active_gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };
    let initial_boost = 23.0;
    let boost_after_pickup = BOOST_MAX_AMOUNT - 3.0;

    for (frame_number, boost_amount, last_boost_amount) in [
        (1, initial_boost, initial_boost),
        (2, boost_after_pickup, initial_boost),
        (3, boost_after_pickup, boost_after_pickup),
        (4, boost_after_pickup, boost_after_pickup),
        (5, boost_after_pickup, boost_after_pickup),
        (6, boost_after_pickup, boost_after_pickup),
    ] {
        calculator
            .update_parts(
                &FrameInfo {
                    frame_number,
                    time: frame_number as f32 / 10.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        boost_amount,
                        last_boost_amount,
                        pad_position,
                    )],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                &LivePlayState::active_play(),
            )
            .expect("boost update should succeed");
    }

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("player stats should be recorded");
    assert_eq!(player_stats.big_pads_collected, 1);
    assert!((player_stats.amount_collected_big - (BOOST_MAX_AMOUNT - initial_boost)).abs() < 0.001);
    assert!((player_stats.overfill_total - initial_boost).abs() < 0.001);
    assert!(
        (player_stats.amount_used - (BOOST_KICKOFF_START_AMOUNT - initial_boost + 3.0)).abs()
            < 0.001
    );

    let events = calculator.pickup_comparison_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].comparison, BoostPickupComparison::Ghost);
    assert_eq!(events[0].pad_type, BoostPickupPadType::Big);
    assert_eq!(events[0].reported_frame, None);
    assert_eq!(events[0].inferred_frame, Some(2));
}

#[test]
fn non_live_boost_increase_is_not_counted_as_ghost_pickup() {
    let mut calculator = BoostCalculator::new();
    let player_id = PlayerId::Steam(1);
    let (pad_position, _) = standard_soccar_boost_pad_layout()
        .iter()
        .find(|(_, size)| *size == BoostPadSize::Big)
        .copied()
        .expect("standard layout should include big pads");
    let post_goal_gameplay = GameplayState {
        game_state: Some(GAME_STATE_GOAL_SCORED_REPLAY),
        ball_has_been_hit: Some(true),
        ..GameplayState::default()
    };

    for (frame_number, boost_amount, last_boost_amount) in [
        (1, 8.0, 8.0),
        (2, 42.0, 8.0),
        (3, 42.0, 42.0),
        (4, 42.0, 42.0),
        (5, 42.0, 42.0),
        (6, 42.0, 42.0),
    ] {
        calculator
            .update_parts(
                &FrameInfo {
                    frame_number,
                    time: frame_number as f32 / 10.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &post_goal_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        boost_amount,
                        last_boost_amount,
                        pad_position,
                    )],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                &LivePlayState::default(),
            )
            .expect("boost update should succeed");
    }

    let player_stats = calculator
        .player_stats()
        .get(&player_id)
        .expect("initial respawn stats should be recorded");
    assert_eq!(player_stats.big_pads_collected, 0);
    assert_eq!(player_stats.amount_collected, 0.0);
    assert!(calculator.pickup_comparison_events().is_empty());
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
            &LivePlayState::active_play(),
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
                    player_position: None,
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                }],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
                        player_position: None,
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    },
                    BoostPadEvent {
                        time: 1.1,
                        frame: 2,
                        pad_id: "small-pad-two".to_string(),
                        player: Some(player_id.clone()),
                        player_position: None,
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    },
                ],
                ..FrameEventsState::default()
            },
            &PlayerVerticalState::default(),
            &LivePlayState::active_play(),
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
