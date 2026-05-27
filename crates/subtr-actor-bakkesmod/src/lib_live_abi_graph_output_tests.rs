use super::*;

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
