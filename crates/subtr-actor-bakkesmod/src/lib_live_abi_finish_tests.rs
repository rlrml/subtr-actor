use super::*;

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
            test_rigid_body(
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
            test_rigid_body(
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
            test_rigid_body(
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
