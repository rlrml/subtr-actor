use super::*;

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
