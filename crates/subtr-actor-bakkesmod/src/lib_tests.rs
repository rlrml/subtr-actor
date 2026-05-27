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

#[path = "lib_live_play_tests.rs"]
mod live_play_tests;

#[path = "lib_finish_tests.rs"]
mod finish_tests;

#[path = "lib_json_export_tests.rs"]
mod json_export_tests;

#[path = "lib_live_event_input_tests.rs"]
mod live_event_input_tests;

#[path = "lib_live_event_history_tests.rs"]
mod live_event_history_tests;

#[path = "lib_live_abi_parity_tests.rs"]
mod live_abi_parity_tests;

#[path = "lib_live_abi_finish_tests.rs"]
mod live_abi_finish_tests;

#[path = "lib_live_abi_stats_module_tests.rs"]
mod live_abi_stats_module_tests;

#[path = "lib_live_abi_graph_output_tests.rs"]
mod live_abi_graph_output_tests;

#[path = "lib_live_abi_frame_tests.rs"]
mod live_abi_frame_tests;

#[path = "lib_live_processor_view_tests.rs"]
mod live_processor_view_tests;

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
