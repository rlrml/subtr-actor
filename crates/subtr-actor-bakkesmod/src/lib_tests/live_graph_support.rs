
struct ExplicitEventFamilyFixture {
    players: [SaPlayerFrame; 2],
    touches: [SaTouchEvent; 1],
    dodge_refreshes: [SaDodgeRefreshedEvent; 1],
    boost_pad_events: [SaBoostPadEvent; 1],
    goals: [SaGoalEvent; 1],
    player_stat_events: [SaPlayerStatEvent; 3],
    demolishes: [SaDemolishEvent; 1],
}

impl ExplicitEventFamilyFixture {
    fn new() -> Self {
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

        Self {
            players: [
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
            ],
            touches: [SaTouchEvent {
                timing: SaEventTiming::default(),
                player_index: 0,
                has_player: 1,
                is_team_0: 1,
                closest_approach_distance: 12.0,
                has_closest_approach_distance: 1,
            }],
            dodge_refreshes: [SaDodgeRefreshedEvent {
                timing: SaEventTiming::default(),
                player_index: 0,
                is_team_0: 1,
                counter_value: 1,
            }],
            boost_pad_events: [SaBoostPadEvent {
                timing: SaEventTiming::default(),
                pad_id: 34,
                kind: SaBoostPadEventKind::PickedUp,
                sequence: 1,
                player_index: 0,
                has_player: 1,
            }],
            goals: [SaGoalEvent {
                timing: SaEventTiming::default(),
                scoring_team_is_team_0: 1,
                player_index: 0,
                has_player: 1,
                team_zero_score: 1,
                has_team_zero_score: 1,
                team_one_score: 0,
                has_team_one_score: 1,
            }],
            player_stat_events: [
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
            ],
            demolishes: [SaDemolishEvent {
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
        }
    }

    fn frames(&self) -> Vec<SaLiveFrame> {
        let mut frames = (1..=3)
            .map(|frame_number| {
                let mut frame = live_frame(
                    frame_number,
                    rigid_body(
                        SaVec3 {
                            x: frame_number as f32 * 25.0,
                            y: 0.0,
                            z: 120.0,
                        },
                        SaVec3::default(),
                    ),
                    &self.players,
                );
                frame.has_live_play = 1;
                frame
            })
            .collect::<Vec<_>>();
        frames[0].touches = self.touches.as_ptr();
        frames[0].touch_count = self.touches.len();
        frames[0].dodge_refreshes = self.dodge_refreshes.as_ptr();
        frames[0].dodge_refresh_count = self.dodge_refreshes.len();
        frames[0].boost_pad_events = self.boost_pad_events.as_ptr();
        frames[0].boost_pad_event_count = self.boost_pad_events.len();
        frames[0].goals = self.goals.as_ptr();
        frames[0].goal_count = self.goals.len();
        frames[0].player_stat_events = self.player_stat_events.as_ptr();
        frames[0].player_stat_event_count = self.player_stat_events.len();
        frames[0].demolishes = self.demolishes.as_ptr();
        frames[0].demolish_count = self.demolishes.len();
        frames
    }
}

fn write_json_file(path: &std::path::Path, value: serde_json::Value) {
    let bytes = serde_json::to_vec(&value).expect("graph dump JSON should serialize");
    std::fs::write(path, bytes)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn validate_graph_dump_with_python(dump_dir: &std::path::Path) {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve");
    let validator = repo_root.join("bakkesmod").join("verify-graph-dump.py");
    let python_candidates = if cfg!(windows) {
        ["python", "python3"]
    } else {
        ["python3", "python"]
    };

    for python in python_candidates {
        let output = match std::process::Command::new(python)
            .arg(&validator)
            .arg(dump_dir)
            .arg("--require-event-history")
            .arg("--require-graph-events")
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("failed to run {python}: {error}"),
        };
        assert!(
            output.status.success(),
            "graph dump validator failed with {python}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    panic!("failed to find python executable for graph dump validator");
}

fn direct_full_graph_events_json_value(frame: &SaLiveFrame) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let players = unsafe {
        if frame.player_count == 0 {
            &[]
        } else {
            slice::from_raw_parts(frame.players, frame.player_count)
        }
    };
    let explicit_events = unsafe { frame_event_slices(frame) }
        .expect("test frame explicit event pointers should be valid");
    sync_live_replay_meta(&mut engine, players)
        .expect("test frame replay metadata should initialize");

    let mut graph = graph_with_all_analysis_nodes();
    graph
        .on_replay_meta(
            engine
                .live_replay_meta
                .as_ref()
                .expect("direct graph replay meta should exist"),
        )
        .expect("direct graph should accept replay metadata");
    let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
    graph
        .evaluate_with_state(&frame_input)
        .expect("direct graph should evaluate live frame input");
    graph.finish().expect("direct graph should finish");
    let events = graph
        .state::<StatsTimelineEventsState>()
        .expect("direct graph should expose timeline events")
        .events
        .clone();
    let bytes = serde_json::to_vec(&events).expect("direct graph events should serialize");
    serde_json::from_slice(&bytes).expect("direct graph events json should be valid")
}

fn live_frame_players(frame: &SaLiveFrame) -> &[SaPlayerFrame] {
    unsafe {
        if frame.player_count == 0 {
            &[]
        } else {
            slice::from_raw_parts(frame.players, frame.player_count)
        }
    }
}

fn direct_full_graph_timeline_events(frames: &[SaLiveFrame]) -> ReplayStatsTimelineEvents {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
    }

    graph.finish().expect("direct graph should finish");
    graph
        .state::<StatsTimelineEventsState>()
        .expect("direct graph should expose timeline events")
        .events
        .clone()
}

fn direct_full_graph_timeline_json_value(frames: &[SaLiveFrame]) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();
    let mut timeline_frames = Vec::new();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
        if let Some(frame) = current_timeline_frame(&graph) {
            record_timeline_frame(&mut timeline_frames, frame);
        }
    }

    graph.finish().expect("direct graph should finish");
    let events = graph
        .state::<StatsTimelineEventsState>()
        .expect("direct graph should expose timeline events")
        .events
        .clone();
    if let Some(frame) = current_timeline_frame(&graph) {
        record_timeline_frame(&mut timeline_frames, frame);
    }
    let bytes = serialize_live_timeline(engine.live_replay_meta.as_ref(), events, timeline_frames);
    serde_json::from_slice(&bytes).expect("direct graph timeline json should be valid")
}

fn direct_full_graph_stats_json_value(frames: &[SaLiveFrame]) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
    }

    graph.finish().expect("direct graph should finish");
    let value = builtin_stats_graph_snapshot_json(&graph, engine.live_replay_meta.as_ref())
        .expect("direct graph stats snapshot should serialize");
    let bytes = serde_json::to_vec(&value).expect("direct graph stats json should serialize");
    serde_json::from_slice(&bytes).expect("direct graph stats json should be valid")
}

fn direct_full_graph_frame_json_value(frames: &[SaLiveFrame]) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
    }

    graph.finish().expect("direct graph should finish");
    let frame = current_timeline_frame(&graph).expect("direct graph should expose frame JSON");
    let bytes = serde_json::to_vec(&frame).expect("direct graph frame should serialize");
    serde_json::from_slice(&bytes).expect("direct graph frame json should be valid")
}

fn direct_full_graph_analysis_node_json_value(
    frames: &[SaLiveFrame],
    node_name: &str,
) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
    }

    graph.finish().expect("direct graph should finish");
    let value = builtin_analysis_node_json(node_name, &graph)
        .unwrap_or_else(|_| panic!("direct graph should serialize node {node_name}"));
    let bytes = serde_json::to_vec(&value).expect("direct graph analysis node should serialize");
    serde_json::from_slice(&bytes).expect("direct graph analysis node json should be valid")
}

fn direct_full_graph_analysis_nodes_json_value(frames: &[SaLiveFrame]) -> serde_json::Value {
    let mut engine = SaEngine::default();
    let mut graph = graph_with_all_analysis_nodes();

    for frame in frames {
        let players = live_frame_players(frame);
        let explicit_events = unsafe { frame_event_slices(frame) }
            .expect("test frame explicit event pointers should be valid");
        let signature = live_replay_meta_signature(players);
        if !engine.live_replay_meta_initialized || engine.live_replay_meta_signature != signature {
            let replay_meta = live_replay_meta(players);
            graph
                .on_replay_meta(&replay_meta)
                .expect("direct graph should accept replay metadata");
            engine.live_replay_meta_initialized = true;
            engine.live_replay_meta = Some(replay_meta);
            engine.live_replay_meta_signature = signature;
        }
        let frame_input = frame_input(&mut engine, frame, players, &explicit_events);
        graph
            .evaluate_with_state(&frame_input)
            .expect("direct graph should evaluate live frame input");
    }

    graph.finish().expect("direct graph should finish");
    let value = callable_analysis_nodes_json(&graph)
        .expect("direct graph should serialize all callable analysis nodes");
    let bytes = serde_json::to_vec(&value).expect("direct graph analysis nodes should serialize");
    serde_json::from_slice(&bytes).expect("direct graph analysis nodes json should be valid")
}

#[derive(Debug, PartialEq, Eq)]
struct MechanicEventSnapshot {
    kind: u32,
    player_index: u32,
    is_team_0: u8,
    frame_number: u64,
    time: u32,
    confidence: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct TeamEventSnapshot {
    kind: u32,
    is_team_0: u8,
    start_frame: u64,
    end_frame: u64,
    start_time: u32,
    end_time: u32,
    attackers: u32,
    defenders: u32,
    confidence: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct GoalContextEventSnapshot {
    frame_number: u64,
    time: u32,
    scoring_team_is_team_0: u8,
    has_scorer: u8,
    scorer_index: u32,
    has_scoring_team_most_back_player: u8,
    scoring_team_most_back_player_index: u32,
    has_defending_team_most_back_player: u8,
    defending_team_most_back_player_index: u32,
    has_ball_position: u8,
    ball_position: (u32, u32, u32),
    has_ball_air_time_before_goal: u8,
    ball_air_time_before_goal: u32,
    goal_buildup: u32,
}

fn mechanic_event_snapshot(event: &SaMechanicEvent) -> MechanicEventSnapshot {
    MechanicEventSnapshot {
        kind: event.kind as u32,
        player_index: event.player_index,
        is_team_0: event.is_team_0,
        frame_number: event.frame_number,
        time: event.time.to_bits(),
        confidence: event.confidence.to_bits(),
    }
}

fn team_event_snapshot(event: &SaTeamEvent) -> TeamEventSnapshot {
    TeamEventSnapshot {
        kind: event.kind as u32,
        is_team_0: event.is_team_0,
        start_frame: event.start_frame,
        end_frame: event.end_frame,
        start_time: event.start_time.to_bits(),
        end_time: event.end_time.to_bits(),
        attackers: event.attackers,
        defenders: event.defenders,
        confidence: event.confidence.to_bits(),
    }
}

fn goal_context_event_snapshot(event: &SaGoalContextEvent) -> GoalContextEventSnapshot {
    GoalContextEventSnapshot {
        frame_number: event.frame_number,
        time: event.time.to_bits(),
        scoring_team_is_team_0: event.scoring_team_is_team_0,
        has_scorer: event.has_scorer,
        scorer_index: event.scorer_index,
        has_scoring_team_most_back_player: event.has_scoring_team_most_back_player,
        scoring_team_most_back_player_index: event.scoring_team_most_back_player_index,
        has_defending_team_most_back_player: event.has_defending_team_most_back_player,
        defending_team_most_back_player_index: event.defending_team_most_back_player_index,
        has_ball_position: event.has_ball_position,
        ball_position: (
            event.ball_position.x.to_bits(),
            event.ball_position.y.to_bits(),
            event.ball_position.z.to_bits(),
        ),
        has_ball_air_time_before_goal: event.has_ball_air_time_before_goal,
        ball_air_time_before_goal: event.ball_air_time_before_goal.to_bits(),
        goal_buildup: event.goal_buildup as u32,
    }
}

fn drain_mechanic_event_snapshots(engine: *mut SaEngine) -> Vec<MechanicEventSnapshot> {
    let mut events = vec![
        SaMechanicEvent {
            kind: SaMechanicKind::SpeedFlip,
            player_index: 0,
            is_team_0: 0,
            frame_number: 0,
            time: 0.0,
            confidence: 0.0,
        };
        256
    ];
    let count =
        unsafe { subtr_actor_bakkesmod_drain_events(engine, events.as_mut_ptr(), events.len()) };
    events
        .iter()
        .take(count)
        .map(mechanic_event_snapshot)
        .collect()
}

fn drain_team_event_snapshots(engine: *mut SaEngine) -> Vec<TeamEventSnapshot> {
    let mut events = vec![
        SaTeamEvent {
            kind: SaTeamEventKind::Rush,
            is_team_0: 0,
            start_frame: 0,
            end_frame: 0,
            start_time: 0.0,
            end_time: 0.0,
            attackers: 0,
            defenders: 0,
            confidence: 0.0,
        };
        256
    ];
    let count = unsafe {
        subtr_actor_bakkesmod_drain_team_events(engine, events.as_mut_ptr(), events.len())
    };
    events.iter().take(count).map(team_event_snapshot).collect()
}

fn drain_goal_context_event_snapshots(engine: *mut SaEngine) -> Vec<GoalContextEventSnapshot> {
    let mut events = vec![
        SaGoalContextEvent {
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
        };
        256
    ];
    let count = unsafe {
        subtr_actor_bakkesmod_drain_goal_context_events(engine, events.as_mut_ptr(), events.len())
    };
    events
        .iter()
        .take(count)
        .map(goal_context_event_snapshot)
        .collect()
}

fn drain_snapshots_from_timeline_events(
    timeline_events: &ReplayStatsTimelineEvents,
) -> (
    Vec<MechanicEventSnapshot>,
    Vec<TeamEventSnapshot>,
    Vec<GoalContextEventSnapshot>,
) {
    let mut pending_events = Vec::new();
    let mut emitted_mechanic_ids = HashSet::new();
    let mut pending_team_events = Vec::new();
    let mut emitted_team_event_ids = HashSet::new();
    let mut pending_goal_context_events = Vec::new();
    let mut emitted_goal_context_ids = HashSet::new();
    push_drainable_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &mut pending_team_events,
        &mut emitted_team_event_ids,
        &mut pending_goal_context_events,
        &mut emitted_goal_context_ids,
        timeline_events,
    );

    (
        pending_events.iter().map(mechanic_event_snapshot).collect(),
        pending_team_events
            .iter()
            .map(team_event_snapshot)
            .collect(),
        pending_goal_context_events
            .iter()
            .map(goal_context_event_snapshot)
            .collect(),
    )
}

fn direct_full_graph_drain_event_snapshots(
    frames: &[SaLiveFrame],
) -> (
    Vec<MechanicEventSnapshot>,
    Vec<TeamEventSnapshot>,
    Vec<GoalContextEventSnapshot>,
) {
    let timeline_events = direct_full_graph_timeline_events(frames);
    drain_snapshots_from_timeline_events(&timeline_events)
}

