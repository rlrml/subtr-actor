#[test]
fn accepts_null_players_when_count_is_zero() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 1,
        time: 0.0,
        dt: 0.0,
        seconds_remaining: 0,
        has_seconds_remaining: 0,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 0,
        has_ball_has_been_hit: 0,
        live_play: 1,
        has_ball: 0,
        ball: SaRigidBody::default(),
        players: ptr::null(),
        player_count: 0,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn rejects_null_players_when_count_is_nonzero() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 1,
        time: 0.0,
        dt: 0.0,
        seconds_remaining: 0,
        has_seconds_remaining: 0,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 0,
        has_ball_has_been_hit: 0,
        live_play: 1,
        has_ball: 0,
        ball: SaRigidBody::default(),
        players: ptr::null(),
        player_count: 1,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, -1);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn rejects_duplicate_live_player_indices() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let players = [
        player_at_index(0, true, SaVec3::default()),
        player_at_index(0, false, SaVec3::default()),
    ];
    let frame = live_frame(1, SaRigidBody::default(), &players);

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, -1);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_updates_analysis_graph_state() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        team_zero_score: 2,
        has_team_zero_score: 1,
        team_one_score: 1,
        has_team_one_score: 1,
        possession_team_is_team_0: 1,
        has_possession_team: 1,
        scored_on_team_is_team_0: 0,
        has_scored_on_team: 1,
        live_play: 1,
        has_ball: 0,
        ball: SaRigidBody::default(),
        players: ptr::null(),
        player_count: 0,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    assert!(engine_ref.live_replay_meta_initialized);
    assert!(engine_ref.live_replay_meta.is_some());
    let frame_info = engine_ref
        .graph
        .state::<FrameInfo>()
        .expect("full analysis graph should expose frame info state");
    engine_ref
        .graph
        .state::<StatsTimelineEventsState>()
        .expect("live graph should expose normalized timeline events state");
    assert_eq!(frame_info.frame_number, 7);
    assert_eq!(frame_info.seconds_remaining, Some(299));
    let gameplay = engine_ref
        .graph
        .state::<GameplayState>()
        .expect("full analysis graph should expose gameplay state");
    assert_eq!(gameplay.current_score(), Some((2, 1)));
    assert_eq!(gameplay.possession_team_is_team_0, Some(true));
    assert_eq!(gameplay.scored_on_team_is_team_0, Some(false));
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_does_not_commit_live_event_history_when_graph_evaluation_fails() {
    struct RequiresStringInputNode {
        state: (),
    }

    impl subtr_actor::stats::analysis_graph::AnalysisNode for RequiresStringInputNode {
        type State = ();

        fn name(&self) -> &'static str {
            "requires_string_input"
        }

        fn dependencies(&self) -> Vec<subtr_actor::stats::analysis_graph::AnalysisDependency> {
            vec![subtr_actor::stats::analysis_graph::AnalysisDependency::required::<String>()]
        }

        fn evaluate(
            &mut self,
            _ctx: &subtr_actor::stats::analysis_graph::AnalysisStateContext<'_>,
        ) -> SubtrActorResult<()> {
            Ok(())
        }

        fn state(&self) -> &Self::State {
            &self.state
        }
    }

    let engine = subtr_actor_bakkesmod_engine_create();
    let engine_ref = unsafe { engine.as_mut().expect("engine should be valid") };
    engine_ref.graph = AnalysisGraph::new()
        .with_input_state_type::<String>()
        .with_node(RequiresStringInputNode { state: () });

    let players = [
        player_at_index(
            0,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 20.0,
            },
        ),
        player_at_index(
            1,
            false,
            SaVec3 {
                x: 200.0,
                y: 0.0,
                z: 20.0,
            },
        ),
    ];
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let dodge_refreshes = [SaDodgeRefreshedEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        counter_value: 7,
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
        has_shot_ball: 0,
        shot_ball: SaRigidBody::default(),
        has_shot_player: 0,
        shot_player: SaRigidBody::default(),
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
            x: 200.0,
            y: 0.0,
            z: 20.0,
        },
        active_duration_seconds: 0.25,
    }];
    let mut frame = live_frame(
        1,
        rigid_body(
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 120.0,
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
        -2
    );
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    assert!(
        engine_ref.live_event_history.touch_events.is_empty(),
        "failed graph evaluation should not commit live touch history"
    );
    assert!(
        engine_ref
            .live_event_history
            .dodge_refreshed_events
            .is_empty(),
        "failed graph evaluation should not commit inferred dodge-refresh history"
    );
    assert!(
        engine_ref.live_event_history.boost_pad_events.is_empty(),
        "failed graph evaluation should not commit boost pad history"
    );
    assert!(
        engine_ref.live_event_history.player_stat_events.is_empty(),
        "failed graph evaluation should not commit player stat history"
    );
    assert!(
        engine_ref.live_event_history.goal_events.is_empty(),
        "failed graph evaluation should not commit goal history"
    );
    assert!(
        engine_ref.live_event_history.demo_events.is_empty(),
        "failed graph evaluation should not commit demolish history"
    );
    assert!(
        engine_ref
            .live_events
            .boost_pad_pickup_sequence_times
            .is_empty(),
        "failed graph evaluation should not commit boost pickup dedupe state"
    );
    assert!(
        engine_ref.live_events.last_goal_event.is_none(),
        "failed graph evaluation should not commit goal dedupe state"
    );
    assert!(
        engine_ref.live_events.known_demolishes.is_empty(),
        "failed graph evaluation should not commit demolish dedupe state"
    );
    assert!(
        engine_ref.live_events.dodge_refresh_counters.is_empty(),
        "failed graph evaluation should not commit dodge-refresh dedupe state"
    );
    assert_eq!(engine_ref.pending_events.len(), 0);
    assert!(engine_ref.pending_team_events.is_empty());
    assert!(engine_ref.pending_goal_context_events.is_empty());
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_graph_contains_every_shared_analysis_node() {
    let mut expected_graph = graph_with_all_analysis_nodes();
    expected_graph
        .resolve()
        .expect("shared graph should resolve");
    let expected_names = expected_graph.node_names().collect::<HashSet<_>>();

    let mut graph = live_analysis_graph();
    graph.resolve().expect("live graph should resolve");
    let live_names = graph.node_names().collect::<HashSet<_>>();
    let builtin_names = builtin_analysis_node_names()
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    for name in expected_names {
        assert!(
            live_names.contains(name),
            "live graph should include shared analysis node {name}"
        );
    }
    for name in &live_names {
        assert!(
            builtin_names.contains(name),
            "live graph node should be callable by builtin name: {name}"
        );
    }
    for name in &builtin_names {
        assert!(
            live_names.contains(name),
            "live graph should include every builtin analysis node: {name}"
        );
    }
    assert!(live_names.contains("stats_timeline_frame"));
    assert!(live_names.contains("stats_timeline_events"));
}

#[test]
fn process_frame_uses_explicit_live_play_state_for_analysis_graph() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 0,
        has_live_play: 1,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::Unknown);
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_derives_live_play_when_not_explicit() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        ball_has_been_hit: 0,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 0,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(
        live_play.gameplay_phase,
        GameplayPhase::KickoffWaitingForTouch
    );
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_treats_sampled_game_state_as_replay_phase_signal() {
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 53;
    const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 67;

    let engine = subtr_actor_bakkesmod_engine_create();
    let kickoff_frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        game_state: GAME_STATE_KICKOFF_COUNTDOWN,
        has_game_state: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &kickoff_frame) },
        0
    );
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::KickoffCountdown);
    assert!(!live_play.is_live_play);
    let gameplay = engine_ref
        .graph
        .state::<GameplayState>()
        .expect("full analysis graph should expose gameplay state");
    assert_eq!(gameplay.game_state, Some(GAME_STATE_KICKOFF_COUNTDOWN));

    let replay_frame = SaLiveFrame {
        frame_number: 8,
        time: 1.6,
        dt: 0.016,
        game_state: GAME_STATE_GOAL_SCORED_REPLAY,
        has_game_state: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        ..SaLiveFrame::default()
    };
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &replay_frame) },
        0
    );
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::PostGoal);
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

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

#[test]
fn exposes_full_timeline_events_json_after_processing_frame() {
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
        players: ptr::null(),
        player_count: 0,
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) },
        0
    );
    let json_len = unsafe { subtr_actor_bakkesmod_events_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_events_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);

    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("events json should be valid");
    assert!(value["events"].is_array());
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_write_events_json(engine, ptr::null_mut(), 10) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_timeline_event_fields_are_classified_for_drain_coverage() {
    let value = serde_json::to_value(ReplayStatsTimelineEvents::default())
        .expect("default events should serialize");
    let fields = value
        .as_object()
        .expect("events should serialize as an object")
        .keys()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let accounted_fields = LIVE_GRAPH_EVENT_FIELD_NAMES
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(
        fields, accounted_fields,
        "new timeline event fields need an explicit live drain/export decision"
    );
}

#[test]
fn exposes_live_graph_info_json() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let value = live_graph_info_json_value(engine);
    assert!(value["dag"]
        .as_str()
        .expect("dag should be a string")
        .contains("stats_timeline_events"));
    let builtin_names = value["builtin_analysis_node_names"]
        .as_array()
        .expect("builtin names should be an array");
    assert!(builtin_names.iter().any(|name| name == "settings"));
    assert!(builtin_names
        .iter()
        .any(|name| name == "continuous_ball_control"));
    // `air_dribble` is a stats module, not an analysis node; its node is `ball_carry`.
    assert!(builtin_names.iter().any(|name| name == "ball_carry"));
    assert!(!builtin_names.iter().any(|name| name == "air_dribble"));
    assert!(builtin_names.iter().any(|name| name == "frame_info"));
    assert!(builtin_names.iter().any(|name| name == "live_play"));
    assert!(builtin_names
        .iter()
        .any(|name| name == "stats_timeline_frame"));
    assert!(builtin_names
        .iter()
        .any(|name| name == "stats_timeline_events"));
    let callable_names = value["callable_analysis_node_names"]
        .as_array()
        .expect("callable names should be an array");
    let callable_name_set = callable_names
        .iter()
        .map(|name| {
            name.as_str()
                .expect("callable names should be strings")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    // `core` and `air_dribble` are stats-module names, not analysis nodes, so they
    // are no longer callable as nodes; their providing nodes are.
    assert!(callable_names.iter().any(|name| name == "match_stats"));
    assert!(callable_names.iter().any(|name| name == "ball_carry"));
    assert!(!callable_names.iter().any(|name| name == "core"));
    assert!(!callable_names.iter().any(|name| name == "air_dribble"));
    assert!(callable_names
        .iter()
        .any(|name| name == "continuous_ball_control"));
    assert_eq!(
        value["callable_analysis_node_names"],
        live_analysis_node_names_json_value(engine),
        "graph info should expose the same callable registry as the names ABI"
    );
    let stats_module_names = value["builtin_stats_module_names"]
        .as_array()
        .expect("stats module names should be an array");
    assert_eq!(stats_module_names.len(), builtin_stats_module_names().len());
    for module_name in builtin_stats_module_names() {
        assert!(
            stats_module_names.iter().any(|name| name == module_name),
            "graph info should expose stats module {module_name}"
        );
    }
    let graph_output_names = value["graph_output_names"]
        .as_array()
        .expect("graph output names should be an array");
    assert_eq!(graph_output_names.len(), LIVE_GRAPH_OUTPUT_NAMES.len());
    for output_name in LIVE_GRAPH_OUTPUT_NAMES {
        assert!(
            graph_output_names.iter().any(|name| name == output_name),
            "graph info should expose graph output {output_name}"
        );
    }
    let graph_event_fields = value["graph_event_field_names"]
        .as_array()
        .expect("graph event field names should be an array");
    assert_eq!(graph_event_fields.len(), LIVE_GRAPH_EVENT_FIELD_NAMES.len());
    for field_name in LIVE_GRAPH_EVENT_FIELD_NAMES {
        assert!(
            graph_event_fields.iter().any(|name| name == field_name),
            "graph info should expose graph event field {field_name}"
        );
    }
    let required_graph_event_fields = value["required_graph_event_field_names"]
        .as_array()
        .expect("required graph event field names should be an array");
    assert_eq!(
        required_graph_event_fields.len(),
        REQUIRED_GRAPH_EVENT_FIELD_NAMES.len()
    );
    for field_name in REQUIRED_GRAPH_EVENT_FIELD_NAMES {
        assert!(
            required_graph_event_fields
                .iter()
                .any(|name| name == field_name),
            "graph info should expose required graph event field {field_name}"
        );
    }
    let event_history_fields = value["event_history_field_names"]
        .as_array()
        .expect("event history field names should be an array");
    assert_eq!(
        event_history_fields.len(),
        LIVE_EVENT_HISTORY_FIELD_NAMES.len()
    );
    for field_name in LIVE_EVENT_HISTORY_FIELD_NAMES {
        assert!(
            event_history_fields.iter().any(|name| name == field_name),
            "graph info should expose event_history field {field_name}"
        );
    }
    let required_event_history_fields = value["required_event_history_field_names"]
        .as_array()
        .expect("required event history field names should be an array");
    assert_eq!(
        required_event_history_fields.len(),
        REQUIRED_EVENT_HISTORY_FIELD_NAMES.len()
    );
    for field_name in REQUIRED_EVENT_HISTORY_FIELD_NAMES {
        assert!(
            required_event_history_fields
                .iter()
                .any(|name| name == field_name),
            "graph info should expose required event_history field {field_name}"
        );
    }
    assert!(
        !required_event_history_fields
            .iter()
            .any(|name| name == "active_demos"),
        "active_demos is current state and should not be required as cumulative history"
    );
    let node_names = value["node_names"]
        .as_array()
        .expect("node names should be an array");
    let node_name_set = node_names
        .iter()
        .map(|name| {
            name.as_str()
                .expect("resolved graph node names should be strings")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    assert!(
        node_name_set.is_subset(&callable_name_set),
        "every resolved graph node reported by graph_info should be callable by name"
    );
    for builtin_name in builtin_analysis_node_names() {
        assert!(
            node_names.iter().any(|name| name == builtin_name),
            "graph info should expose live graph node {builtin_name}"
        );
    }
    assert!(node_names
        .iter()
        .any(|name| name == "continuous_ball_control"));
    assert!(node_names.iter().any(|name| name == "stats_timeline_frame"));
    assert!(node_names
        .iter()
        .any(|name| name == "stats_timeline_events"));
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_write_graph_info_json(engine, ptr::null_mut(), 10) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
