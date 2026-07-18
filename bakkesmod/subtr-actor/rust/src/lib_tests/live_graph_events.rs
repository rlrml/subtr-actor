#[test]
fn exposes_full_stats_timeline_json_after_processing_frames() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let blue_name = std::ffi::CString::new("Blue Live").unwrap();
    let mut players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: -100.0,
            y: -200.0,
            z: 92.75,
        },
    )];
    players[0].player_name = blue_name.as_ptr();

    for (frame_number, time) in [(9, 1.75), (10, 1.766)] {
        let frame = SaLiveFrame {
            frame_number,
            time,
            dt: 0.016,
            seconds_remaining: 298,
            has_seconds_remaining: 1,
            ball_has_been_hit: 1,
            has_ball_has_been_hit: 1,
            live_play: 1,
            players: players.as_ptr(),
            player_count: players.len(),
            ..SaLiveFrame::default()
        };
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
            0
        );
    }

    let json_len = unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_timeline_json(engine, bytes.as_mut_ptr(), bytes.len())
    };
    assert_eq!(written, json_len);

    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("timeline json should be valid");
    assert!(value.get("config").is_some());
    assert!(value.get("events").is_some());
    assert_eq!(value["replay_meta"]["team_zero"][0]["name"], "Blue Live");
    let frames = value["frames"].as_array().expect("frames array");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["frame_number"], 9);
    assert_eq!(frames[1]["frame_number"], 10);
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_write_timeline_json(engine, ptr::null_mut(), 10) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn exposes_stats_collector_module_json_after_processing_frame() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let blue_name = std::ffi::CString::new("Blue Live").unwrap();
    let mut players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: -100.0,
            y: -200.0,
            z: 92.75,
        },
    )];
    players[0].player_name = blue_name.as_ptr();
    let frame = SaLiveFrame {
        frame_number: 9,
        time: 1.75,
        dt: 0.016,
        seconds_remaining: 298,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        players: players.as_ptr(),
        player_count: players.len(),
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    let json_len = unsafe { subtr_actor_bakkesmod_stats_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_stats_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);

    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("stats json should be valid");
    let module_names = value["module_names"]
        .as_array()
        .expect("module names should be an array");
    assert_eq!(module_names.len(), default_stats_module_names().len());
    for module_name in default_stats_module_names() {
        assert!(
            module_names.iter().any(|name| name == module_name),
            "stats json should expose stats module {module_name}"
        );
        assert!(
            value["modules"].get(module_name).is_some(),
            "stats json should include module payload for {module_name}"
        );
    }
    assert!(value["config"].get("positioning").is_some());
    assert!(value["modules"].get("core").is_some());
    assert!(value["modules"].get("boost").is_some());
    assert!(value["modules"].get("demo").is_some());
    assert_eq!(value["frame"]["frame_number"], 9);
    assert_eq!(
        value["frame"]["modules"]["core"]["player_stats"]
            .as_array()
            .expect("core frame player stats should be an array")
            .len(),
        1
    );
    let frame_modules = value["frame"]["modules"]
        .as_object()
        .expect("frame modules should be an object");
    for module_name in default_stats_module_names() {
        assert!(
            frame_modules.contains_key(*module_name),
            "stats frame should include module payload for {module_name}"
        );
    }
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_write_stats_json(engine, ptr::null_mut(), 10) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn exposes_current_timeline_frame_json_after_processing_frame() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let blue_name = std::ffi::CString::new("Blue Live").unwrap();
    let orange_name = std::ffi::CString::new("Orange Live").unwrap();
    let mut players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: -100.0,
                y: -200.0,
                z: 92.75,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 100.0,
                y: 200.0,
                z: 92.75,
            },
        ),
    ];
    players[0].player_name = blue_name.as_ptr();
    players[1].player_name = orange_name.as_ptr();
    let frame = SaLiveFrame {
        frame_number: 9,
        time: 1.75,
        dt: 0.016,
        seconds_remaining: 298,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        players: players.as_ptr(),
        player_count: players.len(),
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    let json_len = unsafe { subtr_actor_bakkesmod_frame_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_frame_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);

    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("frame json should be valid");
    assert_eq!(value["frame_number"], 9);
    assert_eq!(value["seconds_remaining"], 298);
    assert_eq!(value["gameplay_phase"], "active_play");
    assert_eq!(value["players"].as_array().expect("players array").len(), 2);
    assert_eq!(value["players"][0]["name"], "Blue Live");
    assert_eq!(value["players"][0]["is_team_0"], true);
    assert_eq!(value["players"][1]["name"], "Orange Live");
    assert_eq!(value["players"][1]["is_team_0"], false);
    assert!(value.get("team_zero").is_some());
    assert!(value.get("team_one").is_some());
    let team_module_names = [
        "fifty_fifty",
        "possession",
        "ball_half",
        "ball_third",
        "rotation",
        "rush",
        "core",
        "backboard",
        "double_tap",
        "one_timer",
        "pass",
        "ball_carry",
        "air_dribble",
        "boost",
        "bump",
        "half_volley",
        "movement",
        "powerslide",
        "demo",
    ];
    let player_module_names = [
        "core",
        "backboard",
        "ceiling_shot",
        "wall_aerial",
        "wall_aerial_shot",
        "double_tap",
        "one_timer",
        "pass",
        "fifty_fifty",
        "speed_flip",
        "half_flip",
        "half_volley",
        "wavedash",
        "touch",
        "whiff",
        "flick",
        "dodge_reset",
        "ball_carry",
        "air_dribble",
        "boost",
        "bump",
        "movement",
        "positioning",
        "rotation",
        "powerslide",
        "demo",
    ];
    for module_name in team_module_names {
        assert!(
            value["team_zero"].get(module_name).is_some(),
            "typed stats frame should include team_zero.{module_name}"
        );
        assert!(
            value["team_one"].get(module_name).is_some(),
            "typed stats frame should include team_one.{module_name}"
        );
    }
    for module_name in player_module_names {
        assert!(
            value["players"][0].get(module_name).is_some(),
            "typed stats frame should include player module {module_name}"
        );
    }
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_write_frame_json(engine, ptr::null_mut(), 10) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_generates_live_touch_events_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 92.75,
    })];
    let first = live_frame(
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
    let second = live_frame(
        2,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(frame_events.touch_events[0].frame, 2);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_does_not_infer_live_dodge_refreshed_events_from_touch_geometry() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let first = live_frame(
        1,
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
    let second = live_frame(
        2,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 180.0,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert!(frame_events.dodge_refreshed_events.is_empty());
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_dodge_refreshed_events_suppress_inferred_duplicates() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 10.0,
        has_closest_approach_distance: 1,
    }];
    let dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
    }];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 180.0,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();
    frame.dodge_refreshes = dodge_refreshes.as_ptr();
    frame.dodge_refresh_count = dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
    assert_eq!(frame_events.dodge_refreshed_events[0].counter_value, 7);
    assert_eq!(
        frame_events.dodge_refreshed_events[0].player,
        RemoteId::SplitScreen(0)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn duplicate_explicit_live_dodge_refresh_counters_are_suppressed_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let dodge_refreshes = [
        SaDodgeRefreshedEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            counter_value: 7,
        },
        SaDodgeRefreshedEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            counter_value: 7,
        },
    ];
    let mut frame = live_frame(
        1,
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
    frame.dodge_refreshes = dodge_refreshes.as_ptr();
    frame.dodge_refresh_count = dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
    assert_eq!(frame_events.dodge_refreshed_events[0].counter_value, 7);
    assert_eq!(
        frame_events.dodge_refreshed_events[0].player,
        RemoteId::SplitScreen(0)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_dodge_refresh_counters_are_monotonic_for_graph_input() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let first_dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
    }];
    let second_dodge_refreshes = [
        SaDodgeRefreshedEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            counter_value: 7,
        },
        SaDodgeRefreshedEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            is_team_0: 1,
            counter_value: 8,
        },
    ];
    let mut first = live_frame(
        1,
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
    let mut second = live_frame(
        2,
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
    first.dodge_refreshes = first_dodge_refreshes.as_ptr();
    first.dodge_refresh_count = first_dodge_refreshes.len();
    second.dodge_refreshes = second_dodge_refreshes.as_ptr();
    second.dodge_refresh_count = second_dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
    assert_eq!(frame_events.dodge_refreshed_events[0].counter_value, 8);
    assert_eq!(frame_events.dodge_refreshed_events[0].frame, 2);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn stale_explicit_live_dodge_refresh_suppresses_inferred_duplicate() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let first_dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
    }];
    let stale_dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
    }];
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
                z: 180.0,
            },
            SaVec3::default(),
        ),
        &players,
    );
    let mut second = live_frame(
        2,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 180.0,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );
    first.dodge_refreshes = first_dodge_refreshes.as_ptr();
    first.dodge_refresh_count = first_dodge_refreshes.len();
    second.touches = touches.as_ptr();
    second.touch_count = touches.len();
    second.dodge_refreshes = stale_dodge_refreshes.as_ptr();
    second.dodge_refresh_count = stale_dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &second) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert!(frame_events.dodge_refreshed_events.is_empty());
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_dodge_refreshed_events_feed_live_touch_state() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at(SaVec3 {
        x: 0.0,
        y: 0.0,
        z: 180.0,
    })];
    let dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
    }];
    let mut frame = live_frame(
        1,
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
    frame.dodge_refreshes = dodge_refreshes.as_ptr();
    frame.dodge_refresh_count = dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);

    let touch_state = engine_ref
        .graph
        .state::<TouchState>()
        .expect("full analysis graph should expose touch state");
    assert_eq!(touch_state.touch_events.len(), 1);
    assert_eq!(
        touch_state.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
