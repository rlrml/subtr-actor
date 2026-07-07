#[test]
fn process_frame_preserves_explicit_live_event_timing_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 180.0,
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
    let timing = SaEventTiming {
        frame_number: 4,
        time: 0.4,
        seconds_remaining: 123,
        has_timing: 1,
        has_seconds_remaining: 1,
    };
    let touches = [SaTouchEvent {
        timing,
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
    }];
    let dodge_refreshes = [SaDodgeRefreshedEvent {
        timing,
        player_index: 0,
        is_team_0: 1,
        counter_value: 3,
    }];
    let boost_pad_events = [SaBoostPadEvent {
        timing,
        pad_id: 34,
        kind: SaBoostPadEventKind::PickedUp,
        sequence: 2,
        player_index: 0,
        has_player: 1,
    }];
    let goals = [SaGoalEvent {
        timing,
        scoring_team_is_team_0: 1,
        player_index: 0,
        has_player: 1,
        team_zero_score: 1,
        has_team_zero_score: 1,
        team_one_score: 0,
        has_team_one_score: 1,
    }];
    let player_stat_events = [SaPlayerStatEvent {
        timing,
        player_index: 0,
        is_team_0: 1,
        kind: SaPlayerStatEventKind::Shot,
        has_shot_ball: 1,
        shot_ball: rigid_body(
            SaVec3 {
                x: 300.0,
                y: 100.0,
                z: 120.0,
            },
            SaVec3 {
                x: 1000.0,
                y: 500.0,
                z: 100.0,
            },
        ),
        has_shot_player: 1,
        shot_player: rigid_body(
            SaVec3 {
                x: 240.0,
                y: 90.0,
                z: 92.75,
            },
            SaVec3 {
                x: 800.0,
                y: 300.0,
                z: 0.0,
            },
        ),
    }];
    let demolishes = [SaDemolishEvent {
        timing,
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
    }];
    let mut frame = live_frame(
        9,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 180.0,
            },
            SaVec3::default(),
        ),
        &players,
    );
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();
    frame.dodge_refreshes = dodge_refreshes.as_ptr();
    frame.dodge_refresh_count = dodge_refreshes.len();
    frame.boost_pad_events = boost_pad_events.as_ptr();
    frame.boost_pad_event_count = boost_pad_events.len();
    frame.goals = goals.as_ptr();
    frame.goal_count = goals.len();
    frame.player_stat_events = player_stat_events.as_ptr();
    frame.player_stat_event_count = player_stat_events.len();
    frame.demolishes = demolishes.as_ptr();
    frame.demolish_count = demolishes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events[0].frame, 4);
    assert_eq!(frame_events.touch_events[0].time, 0.4);
    assert!(
        frame_events
            .dodge_refreshed_events
            .iter()
            .any(|event| event.frame == 4 && event.time == 0.4),
        "explicit dodge refresh timing should survive alongside any same-frame inferred event"
    );
    assert_eq!(frame_events.boost_pad_events[0].frame, 4);
    assert_eq!(frame_events.boost_pad_events[0].time, 0.4);
    assert_eq!(frame_events.goal_events[0].frame, 4);
    assert_eq!(frame_events.goal_events[0].time, 0.4);
    assert_eq!(frame_events.player_stat_events[0].frame, 4);
    assert_eq!(frame_events.player_stat_events[0].time, 0.4);
    assert_eq!(frame_events.demo_events[0].frame, 4);
    assert_eq!(frame_events.demo_events[0].time, 0.4);
    assert_eq!(frame_events.demo_events[0].seconds_remaining, 123);
    assert!(
        frame_events.active_demos.is_empty(),
        "stale queued demolish events should not become active at the retry frame"
    );

    let frame_events_node = live_analysis_node_json_value(engine, "frame_events_state");
    assert_eq!(frame_events_node["touch_events"][0]["frame"], 4);
    assert_eq!(frame_events_node["boost_pad_events"][0]["frame"], 4);
    assert_eq!(frame_events_node["goal_events"][0]["frame"], 4);
    assert_eq!(frame_events_node["player_stat_events"][0]["frame"], 4);
    assert_eq!(frame_events_node["demo_events"][0]["frame"], 4);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_timeline_events_match_direct_full_graph_for_same_live_frame() {
    let engine = subtr_actor_bakkesmod_engine_create();
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
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
    }];
    let goals = [SaGoalEvent {
        timing: SaEventTiming::default(),
        scoring_team_is_team_0: 1,
        player_index: 0,
        has_player: 1,
        team_zero_score: 1,
        has_team_zero_score: 1,
        team_one_score: 0,
        has_team_one_score: 1,
    }];
    let player_stat_events = [SaPlayerStatEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        kind: SaPlayerStatEventKind::Shot,
        has_shot_ball: 1,
        shot_ball: rigid_body(
            SaVec3 {
                x: 300.0,
                y: 100.0,
                z: 120.0,
            },
            SaVec3 {
                x: 1000.0,
                y: 500.0,
                z: 100.0,
            },
        ),
        has_shot_player: 1,
        shot_player: rigid_body(
            SaVec3 {
                x: 240.0,
                y: 90.0,
                z: 92.75,
            },
            SaVec3 {
                x: 800.0,
                y: 300.0,
                z: 0.0,
            },
        ),
    }];
    let demolishes = [SaDemolishEvent {
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
    }];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();
    frame.goals = goals.as_ptr();
    frame.goal_count = goals.len();
    frame.player_stat_events = player_stat_events.as_ptr();
    frame.player_stat_event_count = player_stat_events.len();
    frame.demolishes = demolishes.as_ptr();
    frame.demolish_count = demolishes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    assert_eq!(
        live_events_json_value(engine),
        direct_full_graph_events_json_value(&frame),
        "BakkesMod ABI exported events should match the shared full analysis graph for the same live frame input"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_drained_events_match_direct_full_graph_for_same_live_frame() {
    let engine = subtr_actor_bakkesmod_engine_create();
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
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
    }];
    let goals = [SaGoalEvent {
        timing: SaEventTiming::default(),
        scoring_team_is_team_0: 1,
        player_index: 0,
        has_player: 1,
        team_zero_score: 1,
        has_team_zero_score: 1,
        team_one_score: 0,
        has_team_one_score: 1,
    }];
    let player_stat_events = [SaPlayerStatEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        kind: SaPlayerStatEventKind::Shot,
        has_shot_ball: 1,
        shot_ball: rigid_body(
            SaVec3 {
                x: 300.0,
                y: 100.0,
                z: 120.0,
            },
            SaVec3 {
                x: 1000.0,
                y: 500.0,
                z: 100.0,
            },
        ),
        has_shot_player: 1,
        shot_player: rigid_body(
            SaVec3 {
                x: 240.0,
                y: 90.0,
                z: 92.75,
            },
            SaVec3 {
                x: 800.0,
                y: 300.0,
                z: 0.0,
            },
        ),
    }];
    let demolishes = [SaDemolishEvent {
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
    }];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();
    frame.goals = goals.as_ptr();
    frame.goal_count = goals.len();
    frame.player_stat_events = player_stat_events.as_ptr();
    frame.player_stat_event_count = player_stat_events.len();
    frame.demolishes = demolishes.as_ptr();
    frame.demolish_count = demolishes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let actual = (
        drain_mechanic_event_snapshots(engine),
        drain_team_event_snapshots(engine),
        drain_goal_context_event_snapshots(engine),
    );
    let expected = direct_full_graph_drain_event_snapshots(&[frame]);
    assert_eq!(
        actual, expected,
        "BakkesMod ABI drain APIs should expose the same events as the shared full graph for the same live frame input"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_explicit_player_stat_event_kinds_match_direct_full_graph() {
    let engine = subtr_actor_bakkesmod_engine_create();
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
    let shot_ball = rigid_body(
        SaVec3 {
            x: 300.0,
            y: 100.0,
            z: 120.0,
        },
        SaVec3 {
            x: 1000.0,
            y: 500.0,
            z: 100.0,
        },
    );
    let shot_player = rigid_body(
        SaVec3 {
            x: 240.0,
            y: 90.0,
            z: 92.75,
        },
        SaVec3 {
            x: 800.0,
            y: 300.0,
            z: 0.0,
        },
    );
    let player_stat_events = [
        SaPlayerStatEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            kind: SaPlayerStatEventKind::Shot,
            has_shot_ball: 1,
            shot_ball,
            has_shot_player: 1,
            shot_player,
        },
        SaPlayerStatEvent {
            timing: SaEventTiming::default(),
            player_index: 1,
            is_team_0: 0,
            kind: SaPlayerStatEventKind::Save,
            has_shot_ball: 0,
            shot_ball: SaRigidBody::default(),
            has_shot_player: 0,
            shot_player: SaRigidBody::default(),
        },
        SaPlayerStatEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            kind: SaPlayerStatEventKind::Assist,
            has_shot_ball: 0,
            shot_ball: SaRigidBody::default(),
            has_shot_player: 0,
            shot_player: SaRigidBody::default(),
        },
    ];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    frame.player_stat_events = player_stat_events.as_ptr();
    frame.player_stat_event_count = player_stat_events.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let live_events = live_events_json_value(engine);
    assert_eq!(
        live_events,
        direct_full_graph_events_json_value(&frame),
        "explicit player stat events should enter the same full graph path as replay processing"
    );

    let timeline = live_events["events"]
        .as_array()
        .expect("events json events should be an array");
    for (kind, is_team_0) in [("Shot", true), ("Save", false), ("Assist", true)] {
        assert!(
            timeline
                .iter()
                .any(|event| timeline_payload_matches(event, kind, 1, Some(is_team_0), true)),
            "explicit live {kind} player stat events should serialize through the full graph"
        );
    }

    let actual = (
        drain_mechanic_event_snapshots(engine),
        drain_team_event_snapshots(engine),
        drain_goal_context_event_snapshots(engine),
    );
    let expected = direct_full_graph_drain_event_snapshots(&[frame]);
    assert_eq!(
        actual, expected,
        "explicit player stat events should drain from the same full graph timeline"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_finish_is_idempotent_for_exported_graph_views_and_drains() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }

    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    let first_events = live_events_json_value(engine);
    let first_timeline = live_timeline_json_value(engine);
    let first_stats = live_stats_json_value(engine);
    let first_frame = live_frame_json_value(engine);
    let first_drain = (
        drain_mechanic_event_snapshots(engine),
        drain_team_event_snapshots(engine),
        drain_goal_context_event_snapshots(engine),
    );
    assert!(
        first_drain
            .0
            .iter()
            .any(|event| event.kind == SaMechanicKind::BallCarry as u32),
        "first finish should drain finalized delayed ball-carry events"
    );

    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    assert_eq!(live_events_json_value(engine), first_events);
    assert_eq!(live_timeline_json_value(engine), first_timeline);
    assert_eq!(live_stats_json_value(engine), first_stats);
    assert_eq!(live_frame_json_value(engine), first_frame);
    assert_eq!(drain_mechanic_event_snapshots(engine), Vec::new());
    assert_eq!(drain_team_event_snapshots(engine), Vec::new());
    assert_eq!(drain_goal_context_event_snapshots(engine), Vec::new());
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_accepts_frames_after_finish_for_mid_game_dumps() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let first_frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 1,
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first_frame) },
        0
    );
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    assert_eq!(live_frame_json_value(engine)["frame_number"], 7);

    let second_frame = SaLiveFrame {
        frame_number: 8,
        time: 1.516,
        dt: 0.016,
        seconds_remaining: 298,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 1,
        ..SaLiveFrame::default()
    };
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second_frame) },
        0
    );

    let value = live_frame_json_value(engine);
    assert_eq!(value["frame_number"], 8);
    assert_eq!(value["seconds_remaining"], 298);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_timeline_json_matches_direct_full_graph_across_finish() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    assert_eq!(
        live_timeline_json_value(engine),
        direct_full_graph_timeline_json_value(&frames),
        "BakkesMod ABI live timeline JSON should match the shared full graph across multi-frame evaluation and finish"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_stats_json_matches_direct_full_graph_across_finish() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    assert_eq!(
        live_stats_json_value(engine),
        direct_full_graph_stats_json_value(&frames),
        "BakkesMod ABI stats JSON should match the shared full graph across multi-frame evaluation and finish"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_every_builtin_stats_module_by_name() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let stats = live_stats_json_value(engine);
    let modules = stats["modules"]
        .as_object()
        .expect("stats json should expose a modules object");
    for module_name in builtin_stats_module_names() {
        assert_eq!(
            live_stats_module_json_value(engine, module_name),
            modules
                .get(*module_name)
                .cloned()
                .unwrap_or_else(|| panic!("stats snapshot should include {module_name}")),
            "named BakkesMod stats module ABI should match full stats snapshot module {module_name}"
        );
    }

    let unknown = std::ffi::CString::new("not_a_module").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, ptr::null()) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
