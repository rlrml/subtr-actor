#[test]
fn process_frame_accepts_explicit_live_event_arrays_for_graph_input() {
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
    let dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 3,
    }];
    let boost_pad_events = [SaBoostPadEvent {
        timing: SaEventTiming::default(),
        pad_id: 34,
        kind: SaBoostPadEventKind::PickedUp,
        sequence: 2,
        player_index: 0,
        has_player: 1,
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
        active_duration_seconds: 0.0,
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
    let player_frame = engine_ref
        .graph
        .state::<PlayerFrameState>()
        .expect("full analysis graph should expose player frame state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
    assert_eq!(frame_events.boost_pad_events.len(), 1);
    assert_eq!(frame_events.goal_events.len(), 1);
    assert_eq!(frame_events.player_stat_events.len(), 1);
    assert_eq!(frame_events.demo_events.len(), 1);
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(frame_events.boost_pad_events[0].pad_id, "34");
    assert_eq!(frame_events.goal_events[0].team_zero_score, Some(1));
    assert_eq!(
        frame_events.player_stat_events[0]
            .shot
            .as_ref()
            .expect("shot metadata should be populated")
            .ball_position
            .x,
        300.0
    );
    assert_eq!(frame_events.demo_events[0].victim, RemoteId::SplitScreen(1));
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
    let frame_events_node = live_analysis_node_json_value(engine, "frame_events_state");
    assert_eq!(
        frame_events_node["touch_events"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        frame_events_node["dodge_refreshed_events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        frame_events_node["boost_pad_events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        frame_events_node["goal_events"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        frame_events_node["player_stat_events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        frame_events_node["demo_events"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        frame_events_node["active_demos"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        frame_events_node["boost_pad_events"][0]["pad_id"],
        serde_json::json!("34")
    );
    assert_eq!(
        frame_events_node["goal_events"][0]["team_zero_score"],
        serde_json::json!(1)
    );
    assert_eq!(
        frame_events_node["player_stat_events"][0]["kind"],
        serde_json::json!("Shot")
    );
    assert_eq!(
        frame_events_node["demo_events"][0]["victim"],
        serde_json::json!({"SplitScreen": 1})
    );
    assert_eq!(
        live_graph_output_json_value(engine, "analysis_nodes")["frame_events_state"],
        frame_events_node,
        "bulk analysis_nodes output should include the callable frame_events_state payload"
    );
    let event_history = live_graph_output_json_value(engine, "event_history");
    assert_eq!(event_history["touch_events"].as_array().unwrap().len(), 1);
    assert_eq!(
        event_history["dodge_refreshed_events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        event_history["boost_pad_events"].as_array().unwrap().len(),
        1
    );
    assert_eq!(event_history["goal_events"].as_array().unwrap().len(), 1);
    assert_eq!(
        event_history["player_stat_events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(event_history["demo_events"].as_array().unwrap().len(), 1);
    assert_eq!(event_history["active_demos"].as_array().unwrap().len(), 1);
    for field_name in REQUIRED_EVENT_HISTORY_FIELD_NAMES {
        let entries = event_history
            .get(*field_name)
            .unwrap_or_else(|| panic!("event_history output should include {field_name}"))
            .as_array()
            .unwrap_or_else(|| panic!("event_history output {field_name} should be an array"));
        assert!(
            !entries.is_empty(),
            "required event_history field {field_name} should be nonzero after explicit live event arrays"
        );
    }
    let mut drained_event_buffer = [SaMechanicEvent {
        kind: SaMechanicKind::Shot,
        player_index: 0,
        is_team_0: 0,
        frame_number: 0,
        time: 0.0,
        confidence: 0.0,
    }; 64];
    let mut goal_context_events = [SaGoalContextEvent {
        frame_number: 0,
        time: 0.0,
        scoring_team_is_team_0: 0,
        has_scorer: 0,
        scorer_index: 0,
        has_scoring_team_most_back_player: 0,
        scoring_team_most_back_player_index: 0,
        has_defending_team_most_back_player: 0,
        defending_team_most_back_player_index: 0,
        has_ball_position: 0,
        ball_position: SaVec3::default(),
        has_ball_air_time_before_goal: 0,
        ball_air_time_before_goal: 0.0,
        goal_buildup: SaGoalBuildupKind::Other,
    }; 4];
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    let goal_context_count = unsafe {
        subtr_actor_bakkesmod_drain_goal_context_events(
            engine,
            goal_context_events.as_mut_ptr(),
            goal_context_events.len(),
        )
    };
    assert_eq!(goal_context_count, 1);
    assert_eq!(goal_context_events[0].frame_number, 1);
    assert_eq!(goal_context_events[0].scoring_team_is_team_0, 1);
    assert_eq!(goal_context_events[0].has_scorer, 1);
    assert_eq!(goal_context_events[0].scorer_index, 0);
    let json_len = unsafe { subtr_actor_bakkesmod_events_json_len(engine) };
    let mut event_json_bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_events_json(
            engine,
            event_json_bytes.as_mut_ptr(),
            event_json_bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    let finalized_event_json: serde_json::Value =
        serde_json::from_slice(&event_json_bytes).expect("finalized events json should be valid");
    let finalized_timeline = finalized_event_json["timeline"]
        .as_array()
        .expect("finalized events json timeline should be an array");
    assert!(
        finalized_timeline
            .iter()
            .any(|event| event["kind"] == serde_json::json!("Goal")
                && event["frame"] == serde_json::json!(1)),
        "explicit live goal events should serialize finalized goal timeline events"
    );
    let finalized_count = unsafe {
        subtr_actor_bakkesmod_drain_events(
            engine,
            drained_event_buffer.as_mut_ptr(),
            drained_event_buffer.len(),
        )
    };
    let finalized_events = &drained_event_buffer[..finalized_count];
    assert!(
        finalized_events.iter().any(|event| {
            event.kind == SaMechanicKind::Shot && event.player_index == 0 && event.frame_number == 1
        }),
        "explicit live player stat events should drain through the finalized full graph"
    );
    assert!(
        finalized_events.iter().any(|event| {
            event.kind == SaMechanicKind::Demo
                && event.player_index == 0
                && event.frame_number == 1
        }),
        "explicit live demolish events should drain attacker demo events through the finalized full graph"
    );
    assert!(
        finalized_events.iter().any(|event| {
            event.kind == SaMechanicKind::Death
                && event.player_index == 1
                && event.frame_number == 1
        }),
        "explicit live demolish events should drain victim death events through the finalized full graph"
    );
    assert!(
        finalized_events.iter().any(|event| {
            event.kind == SaMechanicKind::Goal && event.player_index == 0 && event.frame_number == 1
        }),
        "explicit live goal events should drain finalized goal events through the full graph"
    );
    assert_eq!(player_frame.players[1].match_goals, Some(1));
    assert_eq!(player_frame.players[1].match_assists, Some(2));
    assert_eq!(player_frame.players[1].match_saves, Some(3));
    assert_eq!(player_frame.players[1].match_shots, Some(4));
    assert_eq!(player_frame.players[1].match_score, Some(101));
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_event_history_output_remains_after_frame_events_advance() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: 2000.0,
            y: 0.0,
            z: 92.75,
        },
    )];
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 10.0,
        has_closest_approach_distance: 1,
    }];
    let mut first = live_frame(
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
    first.touches = touches.as_ptr();
    first.touch_count = touches.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );

    let second = live_frame(
        2,
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
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let frame_events_node = live_analysis_node_json_value(engine, "frame_events_state");
    assert_eq!(
        frame_events_node["touch_events"].as_array().unwrap().len(),
        0,
        "frame_events_state should expose only the current frame's raw events"
    );
    let event_history = live_graph_output_json_value(engine, "event_history");
    assert_eq!(
        event_history["touch_events"].as_array().unwrap().len(),
        1,
        "event_history should preserve raw live events after frame_events_state advances"
    );
    assert_eq!(
        event_history["touch_events"][0]["frame"],
        serde_json::json!(1)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn duplicate_explicit_live_boost_pickup_sequences_are_suppressed_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: 1024.0,
            y: 0.0,
            z: 92.75,
        },
    )];
    let boost_pad_events = [
        SaBoostPadEvent {
            timing: SaEventTiming::default(),
            pad_id: 34,
            kind: SaBoostPadEventKind::PickedUp,
            sequence: 7,
            player_index: 0,
            has_player: 1,
        },
        SaBoostPadEvent {
            timing: SaEventTiming::default(),
            pad_id: 34,
            kind: SaBoostPadEventKind::PickedUp,
            sequence: 7,
            player_index: 0,
            has_player: 1,
        },
    ];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 1024.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    frame.boost_pad_events = boost_pad_events.as_ptr();
    frame.boost_pad_event_count = boost_pad_events.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.boost_pad_events.len(), 1);
    assert_eq!(frame_events.boost_pad_events[0].pad_id, "34");
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn duplicate_explicit_live_demolishes_are_suppressed_for_graph_input() {
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
    let demolishes = [
        SaDemolishEvent {
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
            active_duration_seconds: 3.0,
        },
        SaDemolishEvent {
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
            active_duration_seconds: 3.0,
        },
    ];
    let mut frame = live_frame(
        1,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
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
    assert_eq!(frame_events.demo_events.len(), 1);
    assert_eq!(frame_events.active_demos.len(), 1);
    let demo = engine_ref
        .graph
        .state::<DemoCalculator>()
        .expect("full analysis graph should expose demo calculator state");
    assert_eq!(demo.timeline().len(), 2);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_demolish_can_repeat_after_dedupe_window() {
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
    let first_demolishes = [SaDemolishEvent {
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
        active_duration_seconds: 0.0,
    }];
    let mut first = live_frame(
        1,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    first.demolishes = first_demolishes.as_ptr();
    first.demolish_count = first_demolishes.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );

    let second_demolishes = [SaDemolishEvent {
        timing: SaEventTiming {
            frame_number: 200,
            time: 20.0,
            seconds_remaining: 280,
            has_timing: 1,
            has_seconds_remaining: 1,
        },
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
        active_duration_seconds: 0.0,
    }];
    let mut second = live_frame(
        200,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    second.demolishes = second_demolishes.as_ptr();
    second.demolish_count = second_demolishes.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.demo_events.len(), 1);
    assert_eq!(frame_events.demo_events[0].frame, 200);
    assert_eq!(frame_events.demo_events[0].time, 20.0);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_boost_pickup_sequence_can_repeat_after_respawn_window() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: 1024.0,
            y: 0.0,
            z: 92.75,
        },
    )];
    let first_boost_pad_events = [SaBoostPadEvent {
        timing: SaEventTiming::default(),
        pad_id: 34,
        kind: SaBoostPadEventKind::PickedUp,
        sequence: 7,
        player_index: 0,
        has_player: 1,
    }];
    let mut first = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 1024.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    first.boost_pad_events = first_boost_pad_events.as_ptr();
    first.boost_pad_event_count = first_boost_pad_events.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );

    let second_boost_pad_events = [SaBoostPadEvent {
        timing: SaEventTiming {
            frame_number: 50,
            time: 5.0,
            seconds_remaining: 295,
            has_timing: 1,
            has_seconds_remaining: 1,
        },
        pad_id: 34,
        kind: SaBoostPadEventKind::PickedUp,
        sequence: 7,
        player_index: 0,
        has_player: 1,
    }];
    let mut second = live_frame(
        50,
        rigid_body(
            SaVec3 {
                x: 1024.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3::default(),
        ),
        &players,
    );
    second.boost_pad_events = second_boost_pad_events.as_ptr();
    second.boost_pad_event_count = second_boost_pad_events.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.boost_pad_events.len(), 1);
    assert_eq!(frame_events.boost_pad_events[0].frame, 50);
    assert_eq!(frame_events.boost_pad_events[0].time, 5.0);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn duplicate_explicit_live_goal_events_are_suppressed_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 92.75,
        },
    )];
    let goals = [
        SaGoalEvent {
            timing: SaEventTiming::default(),
            scoring_team_is_team_0: 1,
            player_index: 0,
            has_player: 1,
            team_zero_score: 1,
            has_team_zero_score: 1,
            team_one_score: 0,
            has_team_one_score: 1,
        },
        SaGoalEvent {
            timing: SaEventTiming::default(),
            scoring_team_is_team_0: 1,
            player_index: 0,
            has_player: 1,
            team_zero_score: 1,
            has_team_zero_score: 1,
            team_one_score: 0,
            has_team_one_score: 1,
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
    frame.goals = goals.as_ptr();
    frame.goal_count = goals.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.goal_events.len(), 1);
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    let event_json = live_events_json_value(engine);
    let goal_count = event_json["timeline"]
        .as_array()
        .expect("events json timeline should be an array")
        .iter()
        .filter(|event| event["kind"] == serde_json::json!("Goal"))
        .count();
    assert_eq!(goal_count, 1);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

