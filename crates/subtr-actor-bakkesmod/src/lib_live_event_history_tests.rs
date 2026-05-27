use super::*;

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
        test_rigid_body(
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
        test_rigid_body(
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
        test_rigid_body(
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
        test_rigid_body(SaVec3::default(), SaVec3::default()),
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
        test_rigid_body(SaVec3::default(), SaVec3::default()),
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
        test_rigid_body(SaVec3::default(), SaVec3::default()),
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
        test_rigid_body(
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
        test_rigid_body(
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
        test_rigid_body(
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
        shot_ball: test_rigid_body(
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
        shot_player: test_rigid_body(
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
        test_rigid_body(
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
