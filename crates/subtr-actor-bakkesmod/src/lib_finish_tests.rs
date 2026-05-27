use super::*;

#[test]
fn finish_refreshes_exported_graph_views() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 1,
        players: ptr::null(),
        player_count: 0,
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    assert!(unsafe { subtr_actor_bakkesmod_events_json_len(engine) } > 0);
    assert!(unsafe { subtr_actor_bakkesmod_frame_json_len(engine) } > 0);
    assert!(unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) } > 0);
    assert!(unsafe { subtr_actor_bakkesmod_stats_json_len(engine) } > 0);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn finish_drains_finalized_live_ball_carry_events() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let mut events = [SaMechanicEvent {
        kind: SaMechanicKind::SpeedFlip,
        player_index: 0,
        is_team_0: 0,
        frame_number: 0,
        time: 0.0,
        confidence: 0.0,
    }; 8];

    for frame_number in 1..=12 {
        let players = [player_at(SaVec3 {
            x: frame_number as f32 * 20.0,
            y: 0.0,
            z: 20.0,
        })];
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
            &players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            let touches = [SaTouchEvent {
                timing: SaEventTiming::default(),
                player_index: 0,
                has_player: 1,
                is_team_0: 1,
                closest_approach_distance: 0.0,
                has_closest_approach_distance: 1,
            }];
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
            assert_eq!(
                unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
                0
            );
        } else {
            assert_eq!(
                unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
                0
            );
        }
    }

    let pre_finish_count =
        unsafe { subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len()) };
    assert!(events[..pre_finish_count]
        .iter()
        .all(|event| event.kind != SaMechanicKind::BallCarry));
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);
    let count =
        unsafe { subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len()) };
    assert!(
        events[..count].iter().any(|event| {
            event.kind == SaMechanicKind::BallCarry
                && event.player_index == 0
                && event.is_team_0 == 1
        }),
        "finish should drain the finalized ball-carry event"
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn finish_rejects_null_engine() {
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(ptr::null_mut()) }, -1);
}

#[test]
fn drains_pending_team_events_through_abi() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let engine_ref = unsafe { engine.as_mut().expect("engine should be valid") };
    engine_ref.pending_team_events.push(SaTeamEvent {
        kind: SaTeamEventKind::Rush,
        is_team_0: 1,
        start_frame: 4,
        end_frame: 9,
        start_time: 0.4,
        end_time: 0.9,
        attackers: 3,
        defenders: 1,
        confidence: 1.0,
    });
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_pending_team_event_count(engine) },
        1
    );

    let mut events = [SaTeamEvent {
        kind: SaTeamEventKind::Rush,
        is_team_0: 0,
        start_frame: 0,
        end_frame: 0,
        start_time: 0.0,
        end_time: 0.0,
        attackers: 0,
        defenders: 0,
        confidence: 0.0,
    }];
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_drain_team_events(engine, events.as_mut_ptr(), 1) },
        1
    );
    assert_eq!(events[0].kind, SaTeamEventKind::Rush);
    assert_eq!(events[0].is_team_0, 1);
    assert_eq!(events[0].attackers, 3);
    assert_eq!(events[0].defenders, 1);
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_pending_team_event_count(engine) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_drain_team_events(engine, ptr::null_mut(), 1) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn drains_pending_goal_context_events_through_abi() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let engine_ref = unsafe { engine.as_mut().expect("engine should be valid") };
    engine_ref
        .pending_goal_context_events
        .push(SaGoalContextEvent {
            frame_number: 9,
            time: 0.9,
            scoring_team_is_team_0: 0,
            has_scorer: 1,
            scorer_index: 1,
            has_scoring_team_most_back_player: 1,
            scoring_team_most_back_player_index: 1,
            has_defending_team_most_back_player: 1,
            defending_team_most_back_player_index: 0,
            has_ball_position: 1,
            ball_position: SaVec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            has_ball_air_time_before_goal: 1,
            ball_air_time_before_goal: 1.25,
            goal_buildup: SaGoalBuildupKind::CounterAttack,
        });
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_pending_goal_context_event_count(engine) },
        1
    );

    let mut events = [SaGoalContextEvent {
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
    }];
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_drain_goal_context_events(engine, events.as_mut_ptr(), 1) },
        1
    );
    assert_eq!(events[0].frame_number, 9);
    assert_eq!(events[0].scorer_index, 1);
    assert_eq!(events[0].goal_buildup, SaGoalBuildupKind::CounterAttack);
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_pending_goal_context_event_count(engine) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_drain_goal_context_events(engine, ptr::null_mut(), 1) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
