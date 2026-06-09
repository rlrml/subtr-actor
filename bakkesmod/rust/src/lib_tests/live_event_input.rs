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
        &player_state(&players),
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

#[test]
fn live_demolish_events_keep_active_demo_state_until_expiration() {
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

    let mut first = live_frame(
        1,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    first.demolishes = demolishes.as_ptr();
    first.demolish_count = demolishes.len();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &first) },
        0
    );

    let second = live_frame(
        2,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
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
    assert_eq!(frame_events.demo_events.len(), 0);
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
    let demo = engine_ref
        .graph
        .state::<DemoCalculator>()
        .expect("full analysis graph should expose demo calculator state");
    assert_eq!(demo.timeline().len(), 2);

    let fourth = live_frame(
        4,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &fourth) },
        0
    );
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert!(frame_events.active_demos.is_empty());
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_touch_marks_kickoff_waiting_frame_as_active_play() {
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
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
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
    frame.ball_has_been_hit = 0;
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(live_play.is_live_play);
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_touch_marks_stale_kickoff_countdown_frame_as_active_play() {
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 53;

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
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
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
    frame.game_state = GAME_STATE_KICKOFF_COUNTDOWN;
    frame.has_game_state = 1;
    frame.kickoff_countdown_time = 3;
    frame.has_kickoff_countdown_time = 1;
    frame.ball_has_been_hit = 0;
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(live_play.is_live_play);
    let frame_events = engine_ref
        .graph
        .state::<FrameEventsState>()
        .expect("full analysis graph should expose frame events state");
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn explicit_live_dodge_refresh_marks_kickoff_waiting_frame_as_active_play() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [player_at_index(
        0,
        true,
        SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 180.0,
        },
    )];
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
    frame.ball_has_been_hit = 0;
    frame.dodge_refreshes = dodge_refreshes.as_ptr();
    frame.dodge_refresh_count = dodge_refreshes.len();

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(live_play.is_live_play);
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
    assert_eq!(
        frame_events.dodge_refreshed_events[0].player,
        RemoteId::SplitScreen(0)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_feeds_explicit_live_touch_events_to_touch_state() {
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
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 12.0,
        has_closest_approach_distance: 1,
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

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );

    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let touch_state = engine_ref
        .graph
        .state::<TouchState>()
        .expect("full analysis graph should expose touch state");
    assert_eq!(touch_state.touch_events.len(), 1);
    assert_eq!(
        touch_state.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    assert_eq!(
        touch_state.touch_events[0].closest_approach_distance,
        Some(12.0)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn duplicate_explicit_live_touches_are_suppressed_for_graph_input() {
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
    let touches = [
        SaTouchEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            has_player: 1,
            is_team_0: 1,
            closest_approach_distance: 12.0,
            has_closest_approach_distance: 1,
        },
        SaTouchEvent {
            timing: SaEventTiming::default(),
            player_index: 0,
            has_player: 1,
            is_team_0: 1,
            closest_approach_distance: 16.0,
            has_closest_approach_distance: 1,
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
    frame.touches = touches.as_ptr();
    frame.touch_count = touches.len();

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
    assert_eq!(
        frame_events.touch_events[0].closest_approach_distance,
        Some(12.0)
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn rejects_null_explicit_event_pointer_when_count_is_nonzero() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let mut frame = SaLiveFrame {
        frame_number: 1,
        live_play: 1,
        ..SaLiveFrame::default()
    };
    frame.touch_count = 1;

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, -1);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn emits_late_inserted_sorted_timeline_mechanics() {
    let mut pending_events = Vec::new();
    let mut emitted_mechanic_ids = HashSet::new();

    push_mechanic_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &[
            normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
            normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
        ],
    );
    assert_eq!(pending_events.len(), 2);

    pending_events.clear();
    push_mechanic_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &[
            normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
            normalized_mechanic("center:15:30:0", "center", 15, 1.5),
            normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
        ],
    );

    assert_eq!(pending_events.len(), 1);
    assert_eq!(pending_events[0].kind, SaMechanicKind::Center);
    assert_eq!(pending_events[0].frame_number, 15);
    assert_eq!(pending_events[0].time, 1.5);
}

#[test]
fn drains_player_owned_events_from_timeline_events() {
    let mut pending_events = Vec::new();
    let mut emitted_mechanic_ids = HashSet::new();
    let mut pending_team_events = Vec::new();
    let mut emitted_team_event_ids = HashSet::new();
    let mut pending_goal_context_events = Vec::new();
    let mut emitted_goal_context_ids = HashSet::new();
    let timeline_events = ReplayStatsTimelineEvents {
        events: vec![
            timeline_event_envelope(TimelineEvent {
                time: 1.05,
                frame: Some(10),
                kind: TimelineEventKind::Goal,
                player_id: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                is_team_0: Some(true),
            }),
            timeline_event_envelope(TimelineEvent {
                time: 1.06,
                frame: Some(10),
                kind: TimelineEventKind::Shot,
                player_id: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                is_team_0: Some(true),
            }),
            timeline_event_envelope(TimelineEvent {
                time: 1.07,
                frame: Some(10),
                kind: TimelineEventKind::Save,
                player_id: Some(RemoteId::SplitScreen(1)),
                player_position: None,
                is_team_0: Some(false),
            }),
            timeline_event_envelope(TimelineEvent {
                time: 1.08,
                frame: Some(10),
                kind: TimelineEventKind::Assist,
                player_id: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                is_team_0: Some(true),
            }),
            timeline_event_envelope(TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Kill,
                player_id: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                is_team_0: Some(true),
            }),
            timeline_event_envelope(TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Death,
                player_id: Some(RemoteId::SplitScreen(1)),
                player_position: None,
                is_team_0: Some(false),
            }),
            goal_context_event_envelope(GoalContextEvent {
                tags: vec![goal_tag(GoalTagKind::FlickGoal)],
                ..goal_context_event(10, 1.09)
            }),
            normalized_mechanic(
            "speed_flip:15:0",
            "speed_flip",
            15,
            1.5,
            ),
            backboard_event_envelope(backboard_event(11, 1.1)),
            whiff_event_envelope(whiff_event(12, 1.2, 0)),
            boost_pickup_event_envelope(boost_pickup_event(125, 1.25)),
            bump_event_envelope(bump_event(13, 1.3, 0.42)),
            fifty_fifty_event_envelope(fifty_fifty_event(9, 14, 1.4)),
            rush_event_envelope(rush_event(8, 16, 1.6, true)),
        ],
    };

    push_drainable_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &mut pending_team_events,
        &mut emitted_team_event_ids,
        &mut pending_goal_context_events,
        &mut emitted_goal_context_ids,
        &timeline_events,
    );

    assert_eq!(pending_events.len(), 13);
    assert_eq!(pending_events[0].kind, SaMechanicKind::Goal);
    assert_eq!(pending_events[0].frame_number, 10);
    assert_eq!(pending_events[0].player_index, 0);
    assert_eq!(pending_events[1].kind, SaMechanicKind::Shot);
    assert_eq!(pending_events[1].frame_number, 10);
    assert_eq!(pending_events[1].player_index, 0);
    assert_eq!(pending_events[2].kind, SaMechanicKind::Save);
    assert_eq!(pending_events[2].frame_number, 10);
    assert_eq!(pending_events[2].player_index, 1);
    assert_eq!(pending_events[3].kind, SaMechanicKind::Assist);
    assert_eq!(pending_events[3].frame_number, 10);
    assert_eq!(pending_events[3].player_index, 0);
    assert_eq!(pending_events[4].kind, SaMechanicKind::FlickGoal);
    assert_eq!(pending_events[4].time, 1.09);
    assert_eq!(pending_events[4].frame_number, 10);
    assert_eq!(pending_events[4].player_index, 1);
    assert_eq!(pending_events[4].is_team_0, 0);
    assert_eq!(pending_events[4].confidence, 0.72);
    assert_eq!(pending_events[5].kind, SaMechanicKind::Backboard);
    assert_eq!(pending_events[5].frame_number, 11);
    assert_eq!(pending_events[5].player_index, 0);
    assert_eq!(pending_events[6].kind, SaMechanicKind::Whiff);
    assert_eq!(pending_events[6].frame_number, 12);
    assert_eq!(pending_events[6].player_index, 0);
    assert_eq!(pending_events[7].kind, SaMechanicKind::BoostPickup);
    assert_eq!(pending_events[7].frame_number, 125);
    assert_eq!(pending_events[7].player_index, 0);
    assert_eq!(pending_events[8].kind, SaMechanicKind::Bump);
    assert_eq!(pending_events[8].frame_number, 13);
    assert_eq!(pending_events[8].player_index, 0);
    assert_eq!(pending_events[8].confidence, 0.42);
    assert_eq!(pending_events[9].kind, SaMechanicKind::Demo);
    assert_eq!(pending_events[9].time, 1.35);
    assert_eq!(pending_events[9].frame_number, 13);
    assert_eq!(pending_events[9].player_index, 0);
    assert_eq!(pending_events[10].kind, SaMechanicKind::Death);
    assert_eq!(pending_events[10].time, 1.35);
    assert_eq!(pending_events[10].frame_number, 13);
    assert_eq!(pending_events[10].player_index, 1);
    assert_eq!(pending_events[10].is_team_0, 0);
    assert_eq!(pending_events[11].kind, SaMechanicKind::FiftyFifty);
    assert_eq!(pending_events[11].frame_number, 14);
    assert_eq!(pending_events[11].player_index, 1);
    assert_eq!(pending_events[11].is_team_0, 0);
    assert_eq!(pending_events[12].kind, SaMechanicKind::SpeedFlip);
    assert_eq!(pending_team_events.len(), 1);
    assert_eq!(pending_team_events[0].kind, SaTeamEventKind::Rush);
    assert_eq!(pending_team_events[0].is_team_0, 1);
    assert_eq!(pending_team_events[0].start_frame, 8);
    assert_eq!(pending_team_events[0].end_frame, 16);
    assert_eq!(pending_team_events[0].start_time, 1.0);
    assert_eq!(pending_team_events[0].end_time, 1.6);
    assert_eq!(pending_team_events[0].attackers, 3);
    assert_eq!(pending_team_events[0].defenders, 2);
    assert_eq!(pending_goal_context_events.len(), 1);
    assert_eq!(pending_goal_context_events[0].frame_number, 10);
    assert_eq!(pending_goal_context_events[0].time, 1.09);
    assert_eq!(pending_goal_context_events[0].scoring_team_is_team_0, 0);
    assert_eq!(pending_goal_context_events[0].has_scorer, 1);
    assert_eq!(pending_goal_context_events[0].scorer_index, 1);
    assert_eq!(
        pending_goal_context_events[0].has_defending_team_most_back_player,
        1
    );
    assert_eq!(
        pending_goal_context_events[0].defending_team_most_back_player_index,
        0
    );
    assert_eq!(pending_goal_context_events[0].has_ball_position, 1);
    assert_eq!(pending_goal_context_events[0].ball_position.x, 1.0);
    assert_eq!(
        pending_goal_context_events[0].has_ball_air_time_before_goal,
        1
    );
    assert_eq!(
        pending_goal_context_events[0].goal_buildup,
        SaGoalBuildupKind::CounterAttack
    );

    pending_events.clear();
    pending_team_events.clear();
    pending_goal_context_events.clear();
    push_drainable_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &mut pending_team_events,
        &mut emitted_team_event_ids,
        &mut pending_goal_context_events,
        &mut emitted_goal_context_ids,
        &timeline_events,
    );
    assert!(pending_events.is_empty());
    assert!(pending_team_events.is_empty());
    assert!(pending_goal_context_events.is_empty());
}

#[test]
fn maps_normalized_timeline_mechanic_kinds_to_abi_kinds() {
    let expected_shared_graph_kinds = HashSet::from([
        "air_dribble",
        "ball_carry",
        "ceiling_shot",
        "center",
        "double_tap",
        "flick",
        "flip_reset",
        "half_flip",
        "half_volley",
        "musty_flick",
        "one_timer",
        "pass",
        "speed_flip",
        "wall_aerial",
        "wall_aerial_shot",
        "wavedash",
    ]);
    let shared_graph_kinds = STATS_TIMELINE_MECHANIC_KINDS
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(
        shared_graph_kinds, expected_shared_graph_kinds,
        "shared stats timeline mechanic kind set changed; update ABI mapping expectations"
    );
    for &kind in STATS_TIMELINE_MECHANIC_KINDS {
        assert!(
            mechanic_kind(kind).is_some(),
            "BakkesMod ABI mapping must cover shared stats timeline mechanic kind: {kind}"
        );
    }

    assert_eq!(
        mechanic_kind("air_dribble"),
        Some(SaMechanicKind::AirDribble)
    );
    assert_eq!(mechanic_kind("ball_carry"), Some(SaMechanicKind::BallCarry));
    assert_eq!(
        mechanic_kind("ceiling_shot"),
        Some(SaMechanicKind::CeilingShot)
    );
    assert_eq!(mechanic_kind("center"), Some(SaMechanicKind::Center));
    assert_eq!(mechanic_kind("double_tap"), Some(SaMechanicKind::DoubleTap));
    assert_eq!(mechanic_kind("flick"), Some(SaMechanicKind::Flick));
    assert_eq!(mechanic_kind("flip_reset"), Some(SaMechanicKind::FlipReset));
    assert_eq!(mechanic_kind("half_flip"), Some(SaMechanicKind::HalfFlip));
    assert_eq!(
        mechanic_kind("half_volley"),
        Some(SaMechanicKind::HalfVolley)
    );
    assert_eq!(
        mechanic_kind("musty_flick"),
        Some(SaMechanicKind::MustyFlick)
    );
    assert_eq!(mechanic_kind("one_timer"), Some(SaMechanicKind::OneTimer));
    assert_eq!(mechanic_kind("pass"), Some(SaMechanicKind::Pass));
    assert_eq!(mechanic_kind("speed_flip"), Some(SaMechanicKind::SpeedFlip));
    assert_eq!(
        mechanic_kind("wall_aerial"),
        Some(SaMechanicKind::WallAerial)
    );
    assert_eq!(
        mechanic_kind("wall_aerial_shot"),
        Some(SaMechanicKind::WallAerialShot)
    );
    assert_eq!(mechanic_kind("wavedash"), Some(SaMechanicKind::Wavedash));
    assert_eq!(mechanic_kind("unmapped"), None);
}

#[test]
fn maps_timeline_event_kinds_to_abi_kinds() {
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Goal),
        SaMechanicKind::Goal
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Shot),
        SaMechanicKind::Shot
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Save),
        SaMechanicKind::Save
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Assist),
        SaMechanicKind::Assist
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Kill),
        SaMechanicKind::Demo
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Death),
        SaMechanicKind::Death
    );
}

#[test]
fn maps_goal_tag_kinds_to_abi_kinds() {
    assert_eq!(
        goal_tag_kind(GoalTagKind::AerialGoal),
        SaMechanicKind::AerialGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::HighAerialGoal),
        SaMechanicKind::HighAerialGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::LongDistanceGoal),
        SaMechanicKind::LongDistanceGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::OwnHalfGoal),
        SaMechanicKind::OwnHalfGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::EmptyNetGoal),
        SaMechanicKind::EmptyNetGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::CounterAttackGoal),
        SaMechanicKind::CounterAttackGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::SustainedPressureGoal),
        SaMechanicKind::SustainedPressureGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::KickoffGoal),
        SaMechanicKind::KickoffGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::FlickGoal),
        SaMechanicKind::FlickGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::DoubleTapGoal),
        SaMechanicKind::DoubleTapGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::OneTimerGoal),
        SaMechanicKind::OneTimerGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::PassingGoal),
        SaMechanicKind::PassingGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::AirDribbleGoal),
        SaMechanicKind::AirDribbleGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::FlipResetGoal),
        SaMechanicKind::FlipResetGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::HalfVolleyGoal),
        SaMechanicKind::HalfVolleyGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::BumpGoal),
        SaMechanicKind::BumpGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::DemoGoal),
        SaMechanicKind::DemoGoal
    );
}
