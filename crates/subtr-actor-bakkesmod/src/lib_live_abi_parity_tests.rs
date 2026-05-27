use super::*;

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
    let shot_ball = test_rigid_body(
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
    let shot_player = test_rigid_body(
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

    let timeline = live_events["timeline"]
        .as_array()
        .expect("events json timeline should be an array");
    for (kind, is_team_0) in [("Shot", true), ("Save", false), ("Assist", true)] {
        assert!(
            timeline.iter().any(|event| {
                event["kind"] == serde_json::json!(kind)
                    && event["frame"] == serde_json::json!(1)
                    && event["is_team_0"] == serde_json::json!(is_team_0)
                    && !event["player_id"].is_null()
            }),
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
