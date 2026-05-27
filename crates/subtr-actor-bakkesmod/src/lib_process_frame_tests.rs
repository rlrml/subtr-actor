use super::*;

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
        test_rigid_body(
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
        let live_name = builtin_analysis_node_aliases()
            .iter()
            .find_map(|alias| (alias.alias == *name).then_some(alias.node_name))
            .unwrap_or(name);
        assert!(
            live_names.contains(live_name),
            "live graph should include every builtin analysis node or resolved alias: {name}"
        );
    }
    assert!(live_names.contains("stats_timeline_frame"));
    assert!(live_names.contains("stats_timeline_events"));
}
