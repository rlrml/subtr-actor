use super::*;
use std::collections::BTreeSet;
use subtr_actor::stats::analysis_graph::STATS_TIMELINE_MECHANIC_KINDS;
use subtr_actor::{
    BoostPickupActivity, BoostPickupComparison, BoostPickupFieldHalf, BoostPickupPadType,
    DemoCalculator, TouchState, WhiffEventKind,
};

#[path = "lib_header_tests.rs"]
mod header_tests;

#[path = "lib_replay_annotation_tests.rs"]
mod replay_annotation_tests;

#[path = "lib_layout_tests.rs"]
mod layout_tests;

#[path = "lib_process_frame_tests.rs"]
mod process_frame_tests;

fn rigid_body(location: SaVec3, linear_velocity: SaVec3) -> SaRigidBody {
    SaRigidBody {
        location,
        rotation: SaQuat::default(),
        linear_velocity,
        angular_velocity: SaVec3::default(),
        has_linear_velocity: 1,
        has_angular_velocity: 1,
        sleeping: 0,
    }
}

fn live_frame(frame_number: u64, ball: SaRigidBody, players: &[SaPlayerFrame]) -> SaLiveFrame {
    SaLiveFrame {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_ball: 1,
        ball,
        players: players.as_ptr(),
        player_count: players.len(),
        ..SaLiveFrame::default()
    }
}

fn player_at_index(player_index: u32, is_team_0: bool, location: SaVec3) -> SaPlayerFrame {
    SaPlayerFrame {
        player_index,
        player_name: ptr::null(),
        is_team_0: is_team_0 as u8,
        has_rigid_body: 1,
        rigid_body: rigid_body(location, SaVec3::default()),
        boost_amount: 33.0,
        last_boost_amount: 33.0,
        boost_active: 0,
        jump_active: 0,
        double_jump_active: 0,
        dodge_active: 0,
        powerslide_active: 0,
        has_match_stats: 1,
        match_goals: player_index as i32,
        match_assists: player_index as i32 + 1,
        match_saves: player_index as i32 + 2,
        match_shots: player_index as i32 + 3,
        match_score: player_index as i32 + 100,
    }
}

fn player_at(location: SaVec3) -> SaPlayerFrame {
    player_at_index(0, true, location)
}

fn normalized_mechanic(id: &str, kind: &str, frame: usize, time: f32) -> MechanicEvent {
    MechanicEvent {
        id: id.to_owned(),
        kind: kind.to_owned(),
        player_id: RemoteId::SplitScreen(0),
        is_team_0: true,
        timing: MechanicTiming::Moment { frame, time },
        properties: Vec::new(),
    }
}

fn whiff_event(frame: usize, time: f32, player_index: u32) -> WhiffEvent {
    WhiffEvent {
        kind: WhiffEventKind::Whiff,
        time,
        frame,
        resolved_time: time,
        resolved_frame: frame,
        player: RemoteId::SplitScreen(player_index),
        is_team_0: player_index == 0,
        closest_approach_distance: 42.0,
        forward_alignment: 0.7,
        approach_speed: 900.0,
        dodge_active: false,
        aerial: false,
    }
}

fn bump_event(frame: usize, time: f32, confidence: f32) -> BumpEvent {
    BumpEvent {
        time,
        frame,
        initiator: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        initiator_is_team_0: true,
        victim_is_team_0: false,
        is_team_bump: false,
        strength: 800.0,
        confidence,
        contact_distance: 120.0,
        closing_speed: 500.0,
        victim_impulse: 220.0,
        initiator_position: [0.0, 0.0, 0.0],
        victim_position: [100.0, 0.0, 0.0],
    }
}

fn backboard_event(frame: usize, time: f32) -> BackboardBounceEvent {
    BackboardBounceEvent {
        time,
        frame,
        player: RemoteId::SplitScreen(0),
        is_team_0: true,
    }
}

fn boost_pickup_event(frame: usize, time: f32) -> BoostPickupComparisonEvent {
    BoostPickupComparisonEvent {
        comparison: BoostPickupComparison::Both,
        frame,
        time,
        player_id: RemoteId::SplitScreen(0),
        is_team_0: true,
        pad_type: BoostPickupPadType::Big,
        field_half: BoostPickupFieldHalf::Opponent,
        activity: BoostPickupActivity::Active,
        reported_frame: Some(frame),
        reported_time: Some(time),
        inferred_frame: None,
        inferred_time: None,
        boost_before: Some(20.0),
        boost_after: Some(100.0),
    }
}

fn fifty_fifty_event(
    start_frame: usize,
    resolve_frame: usize,
    resolve_time: f32,
) -> FiftyFiftyEvent {
    FiftyFiftyEvent {
        start_time: 1.0,
        start_frame,
        resolve_time,
        resolve_frame,
        is_kickoff: false,
        team_zero_player: Some(RemoteId::SplitScreen(0)),
        team_one_player: Some(RemoteId::SplitScreen(1)),
        team_zero_touch_time: None,
        team_zero_touch_frame: None,
        team_zero_dodge_contact: false,
        team_one_touch_time: None,
        team_one_touch_frame: None,
        team_one_dodge_contact: false,
        team_zero_position: [0.0, 0.0, 0.0],
        team_one_position: [100.0, 0.0, 0.0],
        midpoint: [50.0, 0.0, 0.0],
        plane_normal: [1.0, 0.0, 0.0],
        winning_team_is_team_0: Some(false),
        possession_team_is_team_0: Some(false),
    }
}

fn goal_tag_event(kind: GoalTagKind, scorer: Option<RemoteId>) -> GoalTagEvent {
    GoalTagEvent {
        goal_index: 0,
        time: 1.36,
        frame: 13,
        kind,
        scoring_team_is_team_0: false,
        scorer,
        confidence: 0.72,
        modifiers: Vec::new(),
        evidence: Vec::new(),
    }
}

fn rush_event(start_frame: usize, end_frame: usize, end_time: f32, is_team_0: bool) -> RushEvent {
    RushEvent {
        start_time: 1.0,
        start_frame,
        end_time,
        end_frame,
        is_team_0,
        attackers: 3,
        defenders: 2,
    }
}

fn goal_context_event(frame: usize, time: f32) -> GoalContextEvent {
    GoalContextEvent {
        time,
        frame,
        scoring_team_is_team_0: false,
        scorer: Some(RemoteId::SplitScreen(1)),
        scoring_team_most_back_player: Some(RemoteId::SplitScreen(1)),
        defending_team_most_back_player: Some(RemoteId::SplitScreen(0)),
        ball_position: Some(subtr_actor::GoalContextPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        ball_air_time_before_goal: Some(1.25),
        goal_buildup: GoalBuildupKind::CounterAttack,
        scorer_last_touch: None,
        players: Vec::new(),
    }
}

fn live_events_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_events_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_events_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("events json should be valid")
}

fn live_timeline_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_timeline_json(engine, bytes.as_mut_ptr(), bytes.len())
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("timeline json should be valid")
}

fn live_frame_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_frame_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_frame_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("frame json should be valid")
}

fn live_stats_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_stats_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_stats_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats json should be valid")
}

fn live_graph_info_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_graph_info_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_graph_info_json(engine, bytes.as_mut_ptr(), bytes.len())
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("graph info json should be valid")
}

fn live_stats_module_json_value(engine: *const SaEngine, module_name: &str) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module json should be valid")
}

fn live_stats_module_frame_json_value(
    engine: *const SaEngine,
    module_name: &str,
) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_frame_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_frame_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module frame json should be valid")
}

fn live_stats_module_config_json_value(
    engine: *const SaEngine,
    module_name: &str,
) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_config_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_config_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module config json should be valid")
}

fn live_graph_output_json_value(engine: *const SaEngine, output_name: &str) -> serde_json::Value {
    let output_name =
        std::ffi::CString::new(output_name).expect("output name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_graph_output_json_len(engine, output_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_graph_output_json(
            engine,
            output_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("graph output json should be valid")
}

fn live_analysis_node_json_value(engine: *const SaEngine, node_name: &str) -> serde_json::Value {
    let node_name =
        std::ffi::CString::new(node_name).expect("node name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_analysis_node_json_len(engine, node_name.as_ptr()) };
    assert!(json_len > 0, "analysis node {node_name:?} should have JSON");
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_analysis_node_json(
            engine,
            node_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("analysis node json should be valid")
}

fn live_analysis_node_names_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_analysis_node_names_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_analysis_node_names_json(
            engine,
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("analysis node names json should be valid")
}

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
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
    const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

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
    assert!(value.get("timeline").is_some());
    assert!(value.get("mechanics").is_some());
    assert!(value.get("goal_context").is_some());
    assert!(value.get("boost_pickups").is_some());
    assert!(value.get("bump").is_some());
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
    assert!(builtin_names.iter().any(|name| name == "air_dribble"));
    assert!(builtin_names.iter().any(|name| name == "frame_info"));
    assert!(builtin_names.iter().any(|name| name == "live_play"));
    assert!(builtin_names
        .iter()
        .any(|name| name == "stats_timeline_frame"));
    assert!(builtin_names
        .iter()
        .any(|name| name == "stats_timeline_events"));
    let builtin_aliases = value["builtin_analysis_node_aliases"]
        .as_array()
        .expect("builtin aliases should be an array");
    assert!(builtin_aliases
        .iter()
        .any(|alias| alias["alias"] == "core" && alias["node_name"] == "match_stats"));
    assert!(builtin_aliases
        .iter()
        .any(|alias| alias["alias"] == "air_dribble" && alias["node_name"] == "ball_carry"));
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
    assert!(callable_names.iter().any(|name| name == "core"));
    assert!(callable_names.iter().any(|name| name == "match_stats"));
    assert!(callable_names.iter().any(|name| name == "air_dribble"));
    assert!(callable_names.iter().any(|name| name == "ball_carry"));
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
        let live_name = builtin_analysis_node_aliases()
            .iter()
            .find_map(|alias| (alias.alias == *builtin_name).then_some(alias.node_name))
            .unwrap_or(builtin_name);
        assert!(
            node_names.iter().any(|name| name == live_name),
            "graph info should expose live graph node or resolved alias {builtin_name}"
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
    assert_eq!(module_names.len(), builtin_stats_module_names().len());
    for module_name in builtin_stats_module_names() {
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
    for module_name in builtin_stats_module_names() {
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
        "pressure",
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
        "musty_flick",
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
        active_duration_seconds: 0.25,
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
        active_duration_seconds: 0.25,
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
            rigid_body(
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
            rigid_body(
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
            rigid_body(
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

#[test]
fn live_abi_exposes_every_builtin_stats_module_by_name() {
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
            rigid_body(
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

    let stats = live_stats_json_value(engine);
    let modules = stats["modules"]
        .as_object()
        .expect("stats json should expose a modules object");
    for module_name in builtin_stats_module_names() {
        assert_eq!(
                live_stats_module_json_value(engine, module_name),
                modules
                    .get(*module_name)
                    .cloned()
                    .unwrap_or_else(|| panic!("stats snapshot should include {module_name}")),
                "named BakkesMod stats module ABI should match full stats snapshot module {module_name}"
            );
    }

    let unknown = std::ffi::CString::new("not_a_module").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, ptr::null()) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_every_builtin_stats_module_frame_and_config_by_name() {
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
            rigid_body(
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

    let stats = live_stats_json_value(engine);
    let config = stats["config"]
        .as_object()
        .expect("stats json should expose a config object");
    let frame_modules = stats["frame"]["modules"]
        .as_object()
        .expect("stats json should expose frame modules");
    for module_name in builtin_stats_module_names() {
        assert_eq!(
                live_stats_module_frame_json_value(engine, module_name),
                frame_modules
                    .get(*module_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "named BakkesMod stats module frame ABI should match full stats frame module {module_name}"
            );
        assert_eq!(
                live_stats_module_config_json_value(engine, module_name),
                config
                    .get(*module_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "named BakkesMod stats module config ABI should match full stats config module {module_name}"
            );
    }

    let unknown = std::ffi::CString::new("not_a_module").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_frame_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_config_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_frame_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_config_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_named_graph_outputs() {
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
            rigid_body(
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
        live_graph_output_json_value(engine, "events"),
        live_events_json_value(engine)
    );
    assert_eq!(
        live_graph_output_json_value(engine, "frame"),
        live_frame_json_value(engine)
    );
    assert_eq!(
        live_graph_output_json_value(engine, "timeline"),
        live_timeline_json_value(engine)
    );
    assert_eq!(
        live_graph_output_json_value(engine, "stats"),
        live_stats_json_value(engine)
    );
    let event_history = live_graph_output_json_value(engine, "event_history");
    assert_eq!(event_history["touch_events"].as_array().unwrap().len(), 1);
    assert_eq!(
        event_history["touch_events"][0]["frame"],
        serde_json::json!(1)
    );
    let analysis_nodes = live_graph_output_json_value(engine, "analysis_nodes");
    assert_eq!(
        analysis_nodes,
        direct_full_graph_analysis_nodes_json_value(&frames),
        "named all-node graph output should match the shared full graph"
    );
    let callable_node_names = callable_analysis_node_names(unsafe {
        engine
            .as_ref()
            .expect("engine should remain valid while checking callable node names")
    });
    let analysis_node_keys = analysis_nodes
        .as_object()
        .expect("analysis_nodes output should be an object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        analysis_node_keys,
        callable_node_names.iter().cloned().collect::<BTreeSet<_>>(),
        "bulk analysis_nodes output should contain exactly the callable node-name registry"
    );
    for node_name in callable_node_names {
        assert_eq!(
            analysis_nodes
                .get(&node_name)
                .unwrap_or_else(|| panic!("analysis_nodes should include {node_name}")),
            &live_analysis_node_json_value(engine, &node_name),
            "analysis_nodes output should include the same payload as callable node {node_name}"
        );
    }
    assert_eq!(
        live_graph_output_json_value(engine, "graph_info"),
        live_graph_info_json_value(engine)
    );

    let unknown = std::ffi::CString::new("not_an_output").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_graph_output_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_graph_output_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_graph_output_json_len(engine, ptr::null()) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_every_builtin_analysis_node_by_name() {
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
            rigid_body(
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

    let expected_node_names = callable_analysis_node_names(unsafe {
        engine
            .as_ref()
            .expect("engine should remain valid while checking node names")
    });
    let exposed_node_names = live_analysis_node_names_json_value(engine)
        .as_array()
        .expect("live ABI node-name registry should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("live ABI node names should be strings")
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        exposed_node_names, expected_node_names,
        "live ABI should expose the complete callable node-name registry"
    );
    for node_name in &exposed_node_names {
        let value = live_analysis_node_json_value(engine, node_name);
        assert!(
            !value.is_null(),
            "analysis node {node_name} should expose a JSON payload"
        );
        assert_eq!(
            value,
            direct_full_graph_analysis_node_json_value(&frames, node_name),
            "live analysis node {node_name} should match direct full graph output"
        );
    }
    for alias in builtin_analysis_node_aliases() {
        let value = live_analysis_node_json_value(engine, alias.alias);
        assert!(
            !value.is_null(),
            "analysis node alias {} should expose a JSON payload",
            alias.alias
        );
        assert_eq!(
            value,
            direct_full_graph_analysis_node_json_value(&frames, alias.alias),
            "live analysis node alias {} should match direct full graph output",
            alias.alias
        );
    }
    assert_eq!(
        live_analysis_node_json_value(engine, "core"),
        live_stats_module_json_value(engine, "core")
    );
    assert_eq!(
        live_analysis_node_json_value(engine, "match_stats"),
        live_stats_module_json_value(engine, "core")
    );
    let timeline_events = live_analysis_node_json_value(engine, "stats_timeline_events");
    assert!(timeline_events["timeline"].is_array());
    assert!(timeline_events["mechanics"].is_array());
    let timeline_frame = live_analysis_node_json_value(engine, "stats_timeline_frame");
    assert_eq!(timeline_frame["frame_number"], serde_json::json!(12));
    assert!(timeline_frame["players"].is_array());

    let unknown = std::ffi::CString::new("not_a_node").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_analysis_node_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_analysis_node_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_analysis_node_json_len(engine, ptr::null()) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_analysis_node_names_json_len(ptr::null()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_analysis_node_names_json(engine, ptr::null_mut(), 10)
        },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_every_analysis_node_after_explicit_event_families() {
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
        counter_value: 1,
    }];
    let boost_pad_events = [SaBoostPadEvent {
        timing: SaEventTiming::default(),
        pad_id: 34,
        kind: SaBoostPadEventKind::PickedUp,
        sequence: 1,
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
                &players,
            );
            frame.has_live_play = 1;
            frame
        })
        .collect::<Vec<_>>();
    frames[0].touches = touches.as_ptr();
    frames[0].touch_count = touches.len();
    frames[0].dodge_refreshes = dodge_refreshes.as_ptr();
    frames[0].dodge_refresh_count = dodge_refreshes.len();
    frames[0].boost_pad_events = boost_pad_events.as_ptr();
    frames[0].boost_pad_event_count = boost_pad_events.len();
    frames[0].goals = goals.as_ptr();
    frames[0].goal_count = goals.len();
    frames[0].player_stat_events = player_stat_events.as_ptr();
    frames[0].player_stat_event_count = player_stat_events.len();
    frames[0].demolishes = demolishes.as_ptr();
    frames[0].demolish_count = demolishes.len();

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let exposed_node_names = live_analysis_node_names_json_value(engine)
        .as_array()
        .expect("live ABI node-name registry should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("live ABI node names should be strings")
                .to_owned()
        })
        .collect::<Vec<_>>();
    for node_name in exposed_node_names {
        assert_eq!(
                live_analysis_node_json_value(engine, &node_name),
                direct_full_graph_analysis_node_json_value(&frames, &node_name),
                "live analysis node {node_name} should match direct full graph output after every explicit live event family"
            );
    }
    assert_eq!(
            live_graph_output_json_value(engine, "analysis_nodes"),
            direct_full_graph_analysis_nodes_json_value(&frames),
            "bulk analysis_nodes output should match the direct full graph after every explicit live event family"
        );
    let events = live_graph_output_json_value(engine, "events");
    for field_name in REQUIRED_GRAPH_EVENT_FIELD_NAMES {
        let entries = events
            .get(*field_name)
            .unwrap_or_else(|| panic!("events output should include {field_name}"))
            .as_array()
            .unwrap_or_else(|| panic!("events output {field_name} should be an array"));
        assert!(
                !entries.is_empty(),
                "required graph event field {field_name} should be nonzero after explicit live event families"
            );
    }
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn synthetic_live_graph_dump_passes_bakkesmod_validator() {
    let fixture = ExplicitEventFamilyFixture::new();
    let frames = fixture.frames();
    let engine = subtr_actor_bakkesmod_engine_create();
    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let dump_dir = std::env::temp_dir().join(format!(
        "subtr-actor-bakkesmod-graph-dump-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dump_dir)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", dump_dir.display()));

    write_json_file(
        &dump_dir.join("graph-events.json"),
        live_events_json_value(engine),
    );
    write_json_file(
        &dump_dir.join("graph-frame.json"),
        live_frame_json_value(engine),
    );
    write_json_file(
        &dump_dir.join("graph-timeline.json"),
        live_timeline_json_value(engine),
    );
    write_json_file(
        &dump_dir.join("graph-stats.json"),
        live_stats_json_value(engine),
    );
    write_json_file(
        &dump_dir.join("graph-analysis-nodes.json"),
        live_graph_output_json_value(engine, "analysis_nodes"),
    );
    write_json_file(
        &dump_dir.join("graph-event-history.json"),
        live_graph_output_json_value(engine, "event_history"),
    );
    write_json_file(
        &dump_dir.join("graph-info.json"),
        live_graph_info_json_value(engine),
    );

    validate_graph_dump_with_python(&dump_dir);

    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
    let _ = std::fs::remove_dir_all(&dump_dir);
}

#[test]
fn live_abi_frame_json_matches_direct_full_graph_across_finish() {
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
            rigid_body(
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
            live_frame_json_value(engine),
            direct_full_graph_frame_json_value(&frames),
            "BakkesMod ABI frame JSON should match the shared full graph across multi-frame evaluation and finish"
        );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_processor_view_exposes_sampled_jump_state() {
    let mut player = player_at_index(
        3,
        true,
        SaVec3 {
            x: 0.0,
            y: 0.0,
            z: 120.0,
        },
    );
    player.jump_active = 1;
    player.double_jump_active = 1;
    player.dodge_active = 1;
    let players = [player];
    let frame = live_frame(1, SaRigidBody::default(), &players);
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState::default(),
        &event_history,
    );
    let player_id = RemoteId::SplitScreen(3);

    assert_eq!(view.get_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&player_id).unwrap(), 1);
}

#[test]
fn live_processor_view_satisfies_processor_surface_from_live_frame() {
    let blue_name = std::ffi::CString::new("Blue View").unwrap();
    let orange_name = std::ffi::CString::new("Orange View").unwrap();
    let mut players = [
        player_at_index(
            2,
            true,
            SaVec3 {
                x: -100.0,
                y: 20.0,
                z: 92.75,
            },
        ),
        player_at_index(
            5,
            false,
            SaVec3 {
                x: 120.0,
                y: 40.0,
                z: 92.75,
            },
        ),
    ];
    players[0].player_name = blue_name.as_ptr();
    players[0].boost_amount = 72.0;
    players[0].last_boost_amount = 68.0;
    players[0].boost_active = 1;
    players[0].jump_active = 1;
    players[0].double_jump_active = 1;
    players[0].dodge_active = 1;
    players[0].powerslide_active = 1;
    players[0].rigid_body.linear_velocity = SaVec3 {
        x: 0.0,
        y: 400.0,
        z: 0.0,
    };
    players[1].player_name = orange_name.as_ptr();

    let mut frame = live_frame(
        11,
        rigid_body(
            SaVec3 {
                x: 10.0,
                y: 20.0,
                z: 120.0,
            },
            SaVec3 {
                x: 300.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        &players,
    );
    frame.seconds_remaining = 241;
    frame.game_state = 7;
    frame.has_game_state = 1;
    frame.kickoff_countdown_time = 3;
    frame.has_kickoff_countdown_time = 1;
    frame.team_zero_score = 2;
    frame.has_team_zero_score = 1;
    frame.team_one_score = 4;
    frame.has_team_one_score = 1;
    frame.possession_team_is_team_0 = 1;
    frame.has_possession_team = 1;
    frame.scored_on_team_is_team_0 = 0;
    frame.has_scored_on_team = 1;

    let touch_events = vec![TouchEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: Some(RemoteId::SplitScreen(2)),
        team_is_team_0: true,
        closest_approach_distance: Some(8.0),
    }];
    let dodge_refreshed_events = vec![DodgeRefreshedEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        is_team_0: true,
        counter_value: 9,
    }];
    let boost_pad_events = vec![BoostPadEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        pad_id: "34".to_owned(),
        player: Some(RemoteId::SplitScreen(2)),
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    let player_stat_events = vec![PlayerStatEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        is_team_0: true,
        kind: PlayerStatEventKind::Shot,
        shot: None,
    }];
    let goal_events = vec![GoalEvent {
        time: frame.time,
        frame: frame.frame_number as usize,
        scoring_team_is_team_0: true,
        player: Some(RemoteId::SplitScreen(2)),
        team_zero_score: Some(3),
        team_one_score: Some(4),
    }];
    let demo_events = vec![DemolishInfo {
        frame: frame.frame_number as usize,
        time: frame.time,
        seconds_remaining: frame.seconds_remaining,
        attacker: RemoteId::SplitScreen(2),
        victim: RemoteId::SplitScreen(5),
        attacker_velocity: Vector3f {
            x: 2300.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: Vector3f {
            x: 0.0,
            y: 200.0,
            z: 0.0,
        },
        victim_location: Vector3f {
            x: 120.0,
            y: 40.0,
            z: 92.75,
        },
    }];
    let frame_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(2),
            victim: RemoteId::SplitScreen(5),
        }],
        demo_events,
        boost_pad_events,
        touch_events,
        dodge_refreshed_events,
        player_stat_events,
        goal_events,
    };
    let replay_meta = live_replay_meta(&players);
    let mut event_history = SaLiveEventHistory::default();
    event_history.append_frame_events(&frame_events);
    let view = SaLiveProcessorView::new(
        Some(&replay_meta),
        &frame,
        &players,
        frame_events,
        &event_history,
    );
    let blue_id = RemoteId::SplitScreen(2);
    let orange_id = RemoteId::SplitScreen(5);

    assert_eq!(view.get_replay_meta().unwrap().player_count(), 2);
    assert_eq!(view.player_count(), 2);
    assert_eq!(
        view.iter_player_ids_in_order().cloned().collect::<Vec<_>>(),
        vec![blue_id.clone(), orange_id.clone()]
    );
    assert_eq!(view.current_in_game_team_player_counts(), [1, 1]);
    assert_eq!(view.get_seconds_remaining().unwrap(), 241);
    assert_eq!(view.get_replicated_state_name().unwrap(), 7);
    assert_eq!(view.get_replicated_game_state_time_remaining().unwrap(), 3);
    assert!(view.get_ball_has_been_hit().unwrap());
    assert!(!view.get_ignore_ball_syncing().unwrap());
    assert_eq!(view.get_team_scores().unwrap(), (2, 4));
    assert_eq!(view.get_ball_hit_team_num().unwrap(), 0);
    assert_eq!(view.get_scored_on_team_num().unwrap(), 1);

    assert_eq!(
        view.get_normalized_ball_rigid_body().unwrap().location.z,
        120.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame.time)
            .unwrap()
            .linear_velocity
            .unwrap()
            .x,
        300.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame.time + 0.5)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time, 0.0)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time + 0.5, 0.0)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame.time + 0.5, 0.5)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_normalized_player_rigid_body(&blue_id)
            .unwrap()
            .location
            .x,
        -100.0
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame.time)
            .unwrap()
            .location
            .z,
        92.75
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame.time + 0.5)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time, 0.0)
            .unwrap()
            .location
            .y,
        20.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time + 0.5, 0.0)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame.time + 0.5, 0.5)
            .unwrap()
            .location
            .y,
        20.0
    );

    assert_eq!(view.get_player_name(&blue_id).unwrap(), "Blue View");
    assert_eq!(view.get_player_team_key(&blue_id).unwrap(), "0");
    assert_eq!(view.get_player_team_key(&orange_id).unwrap(), "1");
    assert!(view.get_player_is_team_0(&blue_id).unwrap());
    assert!(!view.get_player_is_team_0(&orange_id).unwrap());
    assert_eq!(
        view.get_player_id_from_car_id(&boxcars::ActorId(2))
            .unwrap(),
        blue_id
    );
    assert!(view
        .get_player_id_from_car_id(&boxcars::ActorId(99))
        .is_err());

    assert_eq!(view.get_player_boost_level(&blue_id).unwrap(), 72.0);
    assert_eq!(view.get_player_last_boost_level(&blue_id).unwrap(), 68.0);
    assert!(
        (view.get_player_boost_percentage(&blue_id).unwrap() - boost_amount_to_percent(72.0)).abs()
            < 1e-6
    );
    assert_eq!(view.get_boost_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&blue_id).unwrap(), 1);
    assert!(view.get_powerslide_active(&blue_id).unwrap());
    assert_eq!(view.get_player_match_goals(&orange_id).unwrap(), 5);
    assert_eq!(view.get_player_match_assists(&orange_id).unwrap(), 6);
    assert_eq!(view.get_player_match_saves(&orange_id).unwrap(), 7);
    assert_eq!(view.get_player_match_shots(&orange_id).unwrap(), 8);
    assert_eq!(view.get_player_match_score(&orange_id).unwrap(), 105);

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(view.demolishes().len(), 1);
    assert_eq!(view.boost_pad_events().len(), 1);
    assert_eq!(view.touch_events().len(), 1);
    assert_eq!(view.dodge_refreshed_events().len(), 1);
    assert_eq!(view.player_stat_events().len(), 1);
    assert_eq!(view.goal_events().len(), 1);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 1);
    assert_eq!(view.current_frame_boost_pad_events().len(), 1);
    assert_eq!(view.current_frame_touch_events().len(), 1);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 1);
    assert_eq!(view.current_frame_player_stat_events().len(), 1);
    assert_eq!(view.current_frame_goal_events().len(), 1);
}

#[test]
fn live_processor_view_exposes_cumulative_history_for_aggregate_inputs() {
    fn sample_events(frame: usize, time: f32) -> FrameEventsState {
        FrameEventsState {
            demo_events: vec![DemolishInfo {
                frame,
                time,
                seconds_remaining: 300,
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
                attacker_velocity: Vector3f {
                    x: 2300.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_location: Vector3f {
                    x: 120.0,
                    y: 0.0,
                    z: 92.75,
                },
            }],
            boost_pad_events: vec![BoostPadEvent {
                time,
                frame,
                pad_id: "34".to_owned(),
                player: Some(RemoteId::SplitScreen(0)),
                kind: BoostPadEventKind::PickedUp {
                    sequence: frame as u8,
                },
            }],
            touch_events: vec![TouchEvent {
                time,
                frame,
                team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                closest_approach_distance: Some(12.0),
            }],
            dodge_refreshed_events: vec![DodgeRefreshedEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                is_team_0: true,
                counter_value: frame as i32,
            }],
            player_stat_events: vec![PlayerStatEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                is_team_0: true,
                kind: PlayerStatEventKind::Shot,
                shot: None,
            }],
            goal_events: vec![GoalEvent {
                time,
                frame,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                team_zero_score: Some(frame as i32),
                team_one_score: Some(0),
            }],
            ..FrameEventsState::default()
        }
    }

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
        3,
        rigid_body(SaVec3::default(), SaVec3::default()),
        &players,
    );
    let previous_events = sample_events(1, 0.0);
    let between_sample_events = sample_events(2, 0.5);
    let current_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(0),
            victim: RemoteId::SplitScreen(1),
        }],
        ..FrameEventsState::default()
    };
    let mut event_history = SaLiveEventHistory::default();
    event_history.append_frame_events(&previous_events);
    event_history.append_frame_events(&between_sample_events);
    let view = SaLiveProcessorView::new(None, &frame, &players, current_events, &event_history);

    assert_eq!(view.demolishes().len(), 2);
    assert_eq!(view.boost_pad_events().len(), 2);
    assert_eq!(view.touch_events().len(), 2);
    assert_eq!(view.dodge_refreshed_events().len(), 2);
    assert_eq!(view.player_stat_events().len(), 2);
    assert_eq!(view.goal_events().len(), 2);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 0);
    assert_eq!(view.current_frame_boost_pad_events().len(), 0);
    assert_eq!(view.current_frame_touch_events().len(), 0);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 0);
    assert_eq!(view.current_frame_player_stat_events().len(), 0);
    assert_eq!(view.current_frame_goal_events().len(), 0);

    let aggregate_input = FrameInput::aggregate(&view, 3, frame.time, frame.dt, 1, 1, 1, 1, 1, 1);
    let aggregate_events = aggregate_input.frame_events_state();
    assert_eq!(aggregate_events.active_demos.len(), 1);
    assert_eq!(
        aggregate_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(aggregate_events.demo_events[0].frame, 2);
    assert_eq!(aggregate_events.boost_pad_events[0].frame, 2);
    assert_eq!(aggregate_events.touch_events[0].frame, 2);
    assert_eq!(aggregate_events.dodge_refreshed_events[0].frame, 2);
    assert_eq!(aggregate_events.player_stat_events[0].frame, 2);
    assert_eq!(aggregate_events.goal_events[0].frame, 2);
}

#[test]
fn live_processor_view_resolves_demo_car_actor_ids() {
    let players = [
        player_at_index(
            2,
            true,
            SaVec3 {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
        ),
        player_at_index(
            5,
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
        &[SaDemolishEvent {
            timing: SaEventTiming::default(),
            attacker_index: 2,
            victim_index: 5,
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
                attacker: RemoteId::SplitScreen(2),
                victim: RemoteId::SplitScreen(5),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].victim_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(5)
    );
    assert_eq!(active_demos[0].attacker_velocity().x, 2300.0);
}

#[test]
fn live_frame_input_can_build_active_demos_from_processor_view() {
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

    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame.time,
        frame.dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );

    let frame_events = input.frame_events_state();
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(
        frame_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
}

#[test]
fn live_processor_view_does_not_treat_inactive_demo_events_as_active() {
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
    let demo_events = vec![DemolishInfo {
        frame: 4,
        time: 0.4,
        seconds_remaining: 299,
        attacker: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        attacker_velocity: Vector3f {
            x: 2300.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        victim_location: Vector3f {
            x: 120.0,
            y: 0.0,
            z: 92.75,
        },
    }];
    let event_history = SaLiveEventHistory::default();
    let view = SaLiveProcessorView::new(
        None,
        &frame,
        &players,
        FrameEventsState {
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    assert!(
        view.get_active_demos().unwrap().is_empty(),
        "historical or expired live demo events should not be reported as active demos"
    );
    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame.time,
        frame.dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );
    let frame_events = input.frame_events_state();
    assert!(frame_events.active_demos.is_empty());
    assert_eq!(frame_events.demo_events.len(), 1);
}

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
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;

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
        timeline: vec![
            TimelineEvent {
                time: 1.05,
                frame: Some(10),
                kind: TimelineEventKind::Goal,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.06,
                frame: Some(10),
                kind: TimelineEventKind::Shot,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.07,
                frame: Some(10),
                kind: TimelineEventKind::Save,
                player_id: Some(RemoteId::SplitScreen(1)),
                is_team_0: Some(false),
            },
            TimelineEvent {
                time: 1.08,
                frame: Some(10),
                kind: TimelineEventKind::Assist,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Kill,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Death,
                player_id: Some(RemoteId::SplitScreen(1)),
                is_team_0: Some(false),
            },
        ],
        goal_context: vec![goal_context_event(10, 1.09)],
        mechanics: vec![normalized_mechanic(
            "speed_flip:15:0",
            "speed_flip",
            15,
            1.5,
        )],
        backboard: vec![backboard_event(11, 1.1)],
        whiff: vec![whiff_event(12, 1.2, 0)],
        boost_pickups: vec![boost_pickup_event(125, 1.25)],
        bump: vec![bump_event(13, 1.3, 0.42)],
        fifty_fifty: vec![fifty_fifty_event(9, 14, 1.4)],
        goal_tags: vec![
            goal_tag_event(GoalTagKind::FlickGoal, Some(RemoteId::SplitScreen(1))),
            goal_tag_event(GoalTagKind::AerialGoal, None),
        ],
        rush: vec![rush_event(8, 16, 1.6, true)],
        ..ReplayStatsTimelineEvents::default()
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
    assert_eq!(pending_events[4].kind, SaMechanicKind::Backboard);
    assert_eq!(pending_events[4].frame_number, 11);
    assert_eq!(pending_events[4].player_index, 0);
    assert_eq!(pending_events[5].kind, SaMechanicKind::Whiff);
    assert_eq!(pending_events[5].frame_number, 12);
    assert_eq!(pending_events[5].player_index, 0);
    assert_eq!(pending_events[6].kind, SaMechanicKind::BoostPickup);
    assert_eq!(pending_events[6].frame_number, 125);
    assert_eq!(pending_events[6].player_index, 0);
    assert_eq!(pending_events[7].kind, SaMechanicKind::Bump);
    assert_eq!(pending_events[7].frame_number, 13);
    assert_eq!(pending_events[7].player_index, 0);
    assert_eq!(pending_events[7].confidence, 0.42);
    assert_eq!(pending_events[8].kind, SaMechanicKind::Demo);
    assert_eq!(pending_events[8].time, 1.35);
    assert_eq!(pending_events[8].frame_number, 13);
    assert_eq!(pending_events[8].player_index, 0);
    assert_eq!(pending_events[9].kind, SaMechanicKind::Death);
    assert_eq!(pending_events[9].time, 1.35);
    assert_eq!(pending_events[9].frame_number, 13);
    assert_eq!(pending_events[9].player_index, 1);
    assert_eq!(pending_events[9].is_team_0, 0);
    assert_eq!(pending_events[10].kind, SaMechanicKind::FlickGoal);
    assert_eq!(pending_events[10].time, 1.36);
    assert_eq!(pending_events[10].frame_number, 13);
    assert_eq!(pending_events[10].player_index, 1);
    assert_eq!(pending_events[10].is_team_0, 0);
    assert_eq!(pending_events[10].confidence, 0.72);
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
}
