use super::*;

#[test]
fn live_processor_view_exposes_sampled_jump_state() {
    let mut player = player_at_index(
        3,
        true,
        SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 120.0,
        },
    );
    player.jump_active = 1;
    player.double_jump_active = 1;
    player.dodge_active = 1;
    let players = [player];
    let frame = live_frame(1, SaRigidBody::default(), &players);
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState::default(),
        &event_history,
    );
    let player_id = RemoteId::SplitScreen(3);

    assert_eq!(view.get_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&player_id).unwrap(), 1);
}

#[test]
fn live_processor_view_satisfies_processor_surface_from_live_frame() {
    let blue_name = std::ffi::CString::new("Blue View").unwrap();
    let orange_name = std::ffi::CString::new("Orange View").unwrap();
    let mut players = [
        player_at_index(
            2,
            true,
            SaVec3 {
                x: -100.0,
                y: 20.0,
                z: 92.75,
            },
        ),
        player_at_index(
            5,
            false,
            SaVec3 {
                x: 120.0,
                y: 40.0,
                z: 92.75,
            },
        ),
    ];
    players[0].player_name = blue_name.as_ptr();
    players[0].boost_amount = 72.0;
    players[0].last_boost_amount = 68.0;
    players[0].boost_active = 1;
    players[0].jump_active = 1;
    players[0].double_jump_active = 1;
    players[0].dodge_active = 1;
    players[0].powerslide_active = 1;
    players[0].rigid_body.linear_velocity = SaVec3 {
        x: 0.0,
        y: 400.0,
        z: 0.0,
    };
    players[1].player_name = orange_name.as_ptr();

    let mut frame = live_frame(
        11,
        rigid_body(
            SaVec3 {
                x: 10.0,
                y: 20.0,
                z: 120.0,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );
    frame.seconds_remaining = 241;
    frame.game_state = 7;
    frame.has_game_state = 1;
    frame.kickoff_countdown_time = 3;
    frame.has_kickoff_countdown_time = 1;
    frame.team_zero_score = 2;
    frame.has_team_zero_score = 1;
    frame.team_one_score = 4;
    frame.has_team_one_score = 1;
    frame.possession_team_is_team_0 = 1;
    frame.has_possession_team = 1;
    frame.scored_on_team_is_team_0 = 0;
    frame.has_scored_on_team = 1;

    let touch_events = vec![TouchEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: Some(RemoteId::SplitScreen(2)),
        team_is_team_0: true,
        closest_approach_distance: Some(8.0),
    }];
    let dodge_refreshed_events = vec![DodgeRefreshedEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        is_team_0: true,
        counter_value: 9,
    }];
    let boost_pad_events = vec![BoostPadEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        pad_id: "34".to_owned(),
        player: Some(RemoteId::SplitScreen(2)),
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    let player_stat_events = vec![PlayerStatEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        is_team_0: true,
        kind: PlayerStatEventKind::Shot,
        shot: None,
    }];
    let goal_events = vec![GoalEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        scoring_team_is_team_0: true,
        player: Some(RemoteId::SplitScreen(2)),
        team_zero_score: Some(3),
        team_one_score: Some(4),
    }];
    let demo_events = vec![DemolishInfo {
        frame: frame.frame_number as usize,
        time: frame.time,
        seconds_remaining: frame.seconds_remaining,
        attacker: RemoteId::SplitScreen(2),
        victim: RemoteId::SplitScreen(5),
        attacker_velocity: Vector3f {
            x: 2300.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: Vector3f {
            x: 0.0,
            y: 200.0,
            z: 0.0,
        },
        victim_location: Vector3f {
            x: 120.0,
            y: 40.0,
            z: 92.75,
        },
    }];
    let frame_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(2),
            victim: RemoteId::SplitScreen(5),
        }],
        demo_events,
        boost_pad_events,
        touch_events,
        dodge_refreshed_events,
        player_stat_events,
        goal_events,
    };
    let replay_meta = live_replay_meta(&players);
    let mut event_history = SaLiveEventHistory::default();
    event_history.append_frame_events(&frame_events);
    let view = SaLiveProcessorView::new(
        Some(&replay_meta),
        &frame,
        &players,
        frame_events,
        &event_history,
    );
    let blue_id = RemoteId::SplitScreen(2);
    let orange_id = RemoteId::SplitScreen(5);

    assert_eq!(view.get_replay_meta().unwrap().player_count(), 2);
    assert_eq!(view.player_count(), 2);
    assert_eq!(
        view.iter_player_ids_in_order().cloned().collect::<Vec<_>>(),
        vec![blue_id.clone(), orange_id.clone()]
    );
    assert_eq!(view.current_in_game_team_player_counts(), [1, 1]);
    assert_eq!(view.get_seconds_remaining().unwrap(), 241);
    assert_eq!(view.get_replicated_state_name().unwrap(), 7);
    assert_eq!(view.get_replicated_game_state_time_remaining().unwrap(), 3);
    assert!(view.get_ball_has_been_hit().unwrap());
    assert!(!view.get_ignore_ball_syncing().unwrap());
    assert_eq!(view.get_team_scores().unwrap(), (2, 4));
    assert_eq!(view.get_ball_hit_team_num().unwrap(), 0);
    assert_eq!(view.get_scored_on_team_num().unwrap(), 1);

    assert_eq!(
        view.get_normalized_ball_rigid_body().unwrap().location.z,
        120.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame.time)
            .unwrap()
            .linear_velocity
            .unwrap()
            .x,
        300.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame.time + 0.5)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time, 0.0)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time + 0.5, 0.0)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time + 0.5, 0.5)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_normalized_player_rigid_body(&blue_id)
            .unwrap()
            .location
            .x,
        -100.0
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame.time)
            .unwrap()
            .location
            .z,
        92.75
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame.time + 0.5)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time, 0.0)
            .unwrap()
            .location
            .y,
        20.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time + 0.5, 0.0)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time + 0.5, 0.5)
            .unwrap()
            .location
            .y,
        20.0
    );

    assert_eq!(view.get_player_name(&blue_id).unwrap(), "Blue View");
    assert_eq!(view.get_player_team_key(&blue_id).unwrap(), "0");
    assert_eq!(view.get_player_team_key(&orange_id).unwrap(), "1");
    assert!(view.get_player_is_team_0(&blue_id).unwrap());
    assert!(!view.get_player_is_team_0(&orange_id).unwrap());
    assert_eq!(
        view.get_player_id_from_car_id(&boxcars::ActorId(2))
            .unwrap(),
        blue_id
    );
    assert!(view
        .get_player_id_from_car_id(&boxcars::ActorId(99))
        .is_err());

    assert_eq!(view.get_player_boost_level(&blue_id).unwrap(), 72.0);
    assert_eq!(view.get_player_last_boost_level(&blue_id).unwrap(), 68.0);
    assert!(
        (view.get_player_boost_percentage(&blue_id).unwrap() - boost_amount_to_percent(72.0)).abs()
            < 1e-6
    );
    assert_eq!(view.get_boost_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&blue_id).unwrap(), 1);
    assert!(view.get_powerslide_active(&blue_id).unwrap());
    assert_eq!(view.get_player_match_goals(&orange_id).unwrap(), 5);
    assert_eq!(view.get_player_match_assists(&orange_id).unwrap(), 6);
    assert_eq!(view.get_player_match_saves(&orange_id).unwrap(), 7);
    assert_eq!(view.get_player_match_shots(&orange_id).unwrap(), 8);
    assert_eq!(view.get_player_match_score(&orange_id).unwrap(), 105);

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(view.demolishes().len(), 1);
    assert_eq!(view.boost_pad_events().len(), 1);
    assert_eq!(view.touch_events().len(), 1);
    assert_eq!(view.dodge_refreshed_events().len(), 1);
    assert_eq!(view.player_stat_events().len(), 1);
    assert_eq!(view.goal_events().len(), 1);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 1);
    assert_eq!(view.current_frame_boost_pad_events().len(), 1);
    assert_eq!(view.current_frame_touch_events().len(), 1);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 1);
    assert_eq!(view.current_frame_player_stat_events().len(), 1);
    assert_eq!(view.current_frame_goal_events().len(), 1);
}

#[test]
fn live_processor_view_exposes_cumulative_history_for_aggregate_inputs() {
    fn sample_events(frame: usize, time: f32) -> FrameEventsState {
        FrameEventsState {
            demo_events: vec![DemolishInfo {
                frame,
                time,
                seconds_remaining: 300,
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
                attacker_velocity: Vector3f {
                    x: 2300.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_location: Vector3f {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            }],
            boost_pad_events: vec![BoostPadEvent {
                time,
                frame,
                pad_id: "34".to_owned(),
                player: Some(RemoteId::SplitScreen(0)),
                kind: BoostPadEventKind::PickedUp {
                    sequence: frame as u8,
                },
            }],
            touch_events: vec![TouchEvent {
                time,
                frame,
                team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                closest_approach_distance: Some(12.0),
            }],
            dodge_refreshed_events: vec![DodgeRefreshedEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                is_team_0: true,
                counter_value: frame as i32,
            }],
            player_stat_events: vec![PlayerStatEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                is_team_0: true,
                kind: PlayerStatEventKind::Shot,
                shot: None,
            }],
            goal_events: vec![GoalEvent {
                time,
                frame,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                team_zero_score: Some(frame as i32),
                team_one_score: Some(0),
            }],
            ..FrameEventsState::default()
        }
    }

    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
        ),
    ];
    let frame = live_frame(
        3,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let previous_events = sample_events(1, 0.0);
    let between_sample_events = sample_events(2, 0.5);
    let current_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(0),
            victim: RemoteId::SplitScreen(1),
        }],
        ..FrameEventsState::default()
    };
    let mut event_history = SaLiveEventHistory::default();
    event_history.append_frame_events(&previous_events);
    event_history.append_frame_events(&between_sample_events);
    let view = SaLiveProcessorView::new(None, &frame, &players, current_events, &event_history);

    assert_eq!(view.demolishes().len(), 2);
    assert_eq!(view.boost_pad_events().len(), 2);
    assert_eq!(view.touch_events().len(), 2);
    assert_eq!(view.dodge_refreshed_events().len(), 2);
    assert_eq!(view.player_stat_events().len(), 2);
    assert_eq!(view.goal_events().len(), 2);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 0);
    assert_eq!(view.current_frame_boost_pad_events().len(), 0);
    assert_eq!(view.current_frame_touch_events().len(), 0);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 0);
    assert_eq!(view.current_frame_player_stat_events().len(), 0);
    assert_eq!(view.current_frame_goal_events().len(), 0);

    let aggregate_input = FrameInput::aggregate(&view, 3, frame.time, frame.dt, 1, 1, 1, 1, 1, 1);
    let aggregate_events = aggregate_input.frame_events_state();
    assert_eq!(aggregate_events.active_demos.len(), 1);
    assert_eq!(
        aggregate_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(aggregate_events.demo_events[0].frame, 2);
    assert_eq!(aggregate_events.boost_pad_events[0].frame, 2);
    assert_eq!(aggregate_events.touch_events[0].frame, 2);
    assert_eq!(aggregate_events.dodge_refreshed_events[0].frame, 2);
    assert_eq!(aggregate_events.player_stat_events[0].frame, 2);
    assert_eq!(aggregate_events.goal_events[0].frame, 2);
}

#[test]
fn live_processor_view_resolves_demo_car_actor_ids() {
    let players = [
        player_at_index(
            2,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            5,
            false,
            SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
        ),
    ];
    let frame = live_frame(
        7,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let frame_info = frame_info(&frame);
    let demo_events = explicit_demolish_events(
        &frame_info,
        &[SaDemolishEvent {
            timing: SaEventTiming::default(),
            attacker_index: 2,
            victim_index: 5,
            attacker_velocity: SaVec3 {
                x: 2300.0,
                y: 0.0,
                z: 0.0,
            },
            victim_velocity: SaVec3::default(),
            victim_location: SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
            active_duration_seconds: 0.25,
        }],
    );
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState {
            active_demos: vec![DemoEventSample {
                attacker: RemoteId::SplitScreen(2),
                victim: RemoteId::SplitScreen(5),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].victim_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(5)
    );
    assert_eq!(active_demos[0].attacker_velocity().x, 2300.0);
}

#[test]
fn live_frame_input_can_build_active_demos_from_processor_view() {
    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
        ),
    ];
    let frame = live_frame(
        7,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let frame_info = frame_info(&frame);
    let demo_events = explicit_demolish_events(
        &frame_info,
        &[SaDemolishEvent {
            timing: SaEventTiming::default(),
            attacker_index: 0,
            victim_index: 1,
            attacker_velocity: SaVec3 {
                x: 2300.0,
                y: 0.0,
                z: 0.0,
            },
            victim_velocity: SaVec3::default(),
            victim_location: SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
            active_duration_seconds: 0.25,
        }],
    );
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState {
            active_demos: vec![DemoEventSample {
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame.time,
        frame.dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );

    let frame_events = input.frame_events_state();
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(
        frame_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
}

#[test]
fn live_processor_view_does_not_treat_inactive_demo_events_as_active() {
    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
        ),
    ];
    let frame = live_frame(
        7,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let demo_events = vec![DemolishInfo {
        frame: 4,
        time: 0.4,
        seconds_remaining: 299,
        attacker: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        attacker_velocity: Vector3f {
            x: 2300.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        victim_location: Vector3f {
            x: 120.0,
            y: 0.0,
            z: 92.75,
        },
    }];
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState {
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    assert!(
        view.get_active_demos().unwrap().is_empty(),
        "historical or expired live demo events should not be reported as active demos"
    );
    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame.time,
        frame.dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );
    let frame_events = input.frame_events_state();
    assert!(frame_events.active_demos.is_empty());
    assert_eq!(frame_events.demo_events.len(), 1);
}

#[test]
fn live_processor_view_frame_input_preserves_live_event_streams() {
    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
        ),
    ];
    let frame = live_frame(
        7,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let frame_info = frame_info(&frame);
    let demo_events = explicit_demolish_events(
        &frame_info,
        &[SaDemolishEvent {
            timing: SaEventTiming::default(),
            attacker_index: 0,
            victim_index: 1,
            attacker_velocity: SaVec3 {
                x: 2300.0,
                y: 0.0,
                z: 0.0,
            },
            victim_velocity: SaVec3::default(),
            victim_location: SaVec3 {
                x: 120.0,
                y: 0.0,
                z: 92.75,
            },
            active_duration_seconds: 0.25,
        }],
    );
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState {
            active_demos: vec![DemoEventSample {
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };
    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame.time,
        frame.dt,
        live_play.clone(),
    );

    let frame_events = input.frame_events_state();
    assert_eq!(frame_events.demo_events.len(), 1);
    assert_eq!(
        frame_events.demo_events[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
    let player_frame = input.player_frame_state();
    assert_eq!(player_frame.players.len(), 2);
    assert_eq!(player_frame.players[1].match_score, Some(101));
    assert_eq!(input.live_play_state(), Some(live_play));
}
