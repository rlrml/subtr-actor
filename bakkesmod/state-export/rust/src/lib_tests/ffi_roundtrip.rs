use std::ffi::CString;

use subtr_actor_live::{Encoding, ServerMessage};
use subtr_actor_live_consumer::LiveClient;

fn cstring(text: &str) -> CString {
    CString::new(text).expect("test strings contain no interior nuls")
}

fn se_rigid_body(x: f32, y: f32, z: f32) -> SeRigidBody {
    SeRigidBody {
        location: SeVec3 { x, y, z },
        rotation: SeQuat {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: SeVec3 {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        },
        angular_velocity: SeVec3 {
            x: 0.0,
            y: 0.5,
            z: 0.0,
        },
        has_linear_velocity: 1,
        has_angular_velocity: 1,
        sleeping: 0,
    }
}

fn se_remote_id(platform: u8, online_id: u64, epic_id: *const c_char) -> SeRemoteId {
    SeRemoteId {
        online_id,
        epic_id,
        splitscreen_index: 3,
        platform,
    }
}

fn full_player(name: *const c_char, remote_id: SeRemoteId) -> SePlayerFrame {
    SePlayerFrame {
        player_index: 0,
        player_name: name,
        is_team_0: 1,
        has_rigid_body: 1,
        rigid_body: se_rigid_body(-1000.0, -4000.0, 17.0),
        boost_amount: 85.0,
        last_boost_amount: 80.0,
        boost_active: 3,
        jump_active: 2,
        double_jump_active: 1,
        dodge_active: 4,
        powerslide_active: 1,
        car_body_id: 23,
        has_car_body_id: 1,
        has_match_stats: 1,
        match_goals: 1,
        match_assists: 2,
        match_saves: 3,
        match_shots: 4,
        match_score: 350,
        has_input: 1,
        input: SeControllerInput {
            throttle: 1.0,
            steer: -0.25,
            pitch: 0.5,
            yaw: -0.5,
            roll: 0.125,
            dodge_forward: 1.0,
            dodge_strafe: -1.0,
            handbrake: 1,
            jump: 0,
            activate_boost: 1,
            holding_boost: 1,
        },
        camera: SeCameraState {
            pitch: 120,
            yaw: 200,
            has_pitch: 1,
            has_yaw: 1,
            ball_cam_active: 1,
            has_ball_cam: 1,
        },
        has_dodge_impulse: 1,
        dodge_impulse: SeVec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        has_dodge_torque: 1,
        dodge_torque: SeVec3 {
            x: -1.0,
            y: 2.5,
            z: 0.0,
        },
        remote_id,
    }
}

#[test]
fn converts_a_fully_populated_frame_to_the_owned_model() {
    let name = cstring("Alpha");
    let epic_id = cstring("epic-abc-123");
    let players = [
        full_player(
            name.as_ptr(),
            se_remote_id(SE_REMOTE_ID_PLATFORM_STEAM, 76561198000000001, ptr_null()),
        ),
        SePlayerFrame {
            player_index: 1,
            is_team_0: 0,
            remote_id: se_remote_id(SE_REMOTE_ID_PLATFORM_EPIC, 0, epic_id.as_ptr()),
            ..SePlayerFrame::default()
        },
    ];
    let touches = [SeTouchEvent {
        timing: SeEventTiming {
            frame_number: 41,
            time: 12.4,
            seconds_remaining: 210,
            has_timing: 1,
            has_seconds_remaining: 1,
        },
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 130.5,
        has_closest_approach_distance: 1,
    }];
    let dodge_refreshes = [SeDodgeRefreshedEvent {
        timing: SeEventTiming::default(),
        player_index: 1,
        is_team_0: 0,
        counter_value: 2,
    }];
    let boost_pad_events = [SeBoostPadEvent {
        timing: SeEventTiming::default(),
        pad_id: 17,
        kind: SeBoostPadEventKind::PickedUp,
        sequence: 5,
        player_index: 0,
        has_player: 1,
    }];
    let goals = [SeGoalEvent {
        timing: SeEventTiming::default(),
        scoring_team_is_team_0: 1,
        player_index: 0,
        has_player: 1,
        team_zero_score: 2,
        has_team_zero_score: 1,
        team_one_score: 1,
        has_team_one_score: 1,
    }];
    let player_stat_events = [SePlayerStatEvent {
        timing: SeEventTiming::default(),
        player_index: 0,
        is_team_0: 1,
        kind: SePlayerStatEventKind::Shot,
        has_shot_ball: 1,
        shot_ball: se_rigid_body(0.0, 4000.0, 500.0),
        has_shot_player: 0,
        shot_player: SeRigidBody::default(),
    }];
    let demolishes = [SeDemolishEvent {
        timing: SeEventTiming::default(),
        attacker_index: 0,
        victim_index: 1,
        attacker_velocity: SeVec3 {
            x: 2200.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: SeVec3 {
            x: -100.0,
            y: 0.0,
            z: 0.0,
        },
        victim_location: SeVec3 {
            x: 0.0,
            y: 1000.0,
            z: 17.0,
        },
        active_duration_seconds: 3.0,
    }];
    let frame = SeFrame {
        frame_number: 42,
        time: 12.5,
        dt: 1.0 / 120.0,
        seconds_remaining: 210,
        has_seconds_remaining: 1,
        game_state: 2,
        has_game_state: 1,
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
        has_scored_on_team: 0,
        live_play: 1,
        has_live_play: 1,
        has_ball: 1,
        ball: se_rigid_body(0.0, 0.0, 92.75),
        players: players.as_ptr(),
        player_count: players.len(),
        touches: touches.as_ptr(),
        touch_count: touches.len(),
        dodge_refreshes: dodge_refreshes.as_ptr(),
        dodge_refresh_count: dodge_refreshes.len(),
        boost_pad_events: boost_pad_events.as_ptr(),
        boost_pad_event_count: boost_pad_events.len(),
        goals: goals.as_ptr(),
        goal_count: goals.len(),
        player_stat_events: player_stat_events.as_ptr(),
        player_stat_event_count: player_stat_events.len(),
        demolishes: demolishes.as_ptr(),
        demolish_count: demolishes.len(),
    };

    let live = unsafe { live_frame_from_abi(&frame) }.expect("conversion should succeed");

    assert_eq!(live.frame_number, 42);
    assert_eq!(live.time, 12.5);
    assert_eq!(live.seconds_remaining, Some(210));
    assert_eq!(live.game_state, Some(2));
    assert_eq!(live.kickoff_countdown_time, None);
    assert_eq!(live.ball_has_been_hit, Some(true));
    assert_eq!(live.team_zero_score, Some(2));
    assert_eq!(live.team_one_score, Some(1));
    assert_eq!(live.possession_team_is_team_0, Some(true));
    assert_eq!(live.scored_on_team_is_team_0, None);
    assert_eq!(live.live_play, Some(true));
    let ball = live.ball.as_ref().expect("ball should convert");
    assert_eq!(ball.location.z, 92.75);
    assert_eq!(
        ball.linear_velocity,
        Some(Vector3f {
            x: 100.0,
            y: 0.0,
            z: 0.0
        })
    );

    assert_eq!(live.players.len(), 2);
    let alpha = &live.players[0];
    assert_eq!(alpha.name.as_deref(), Some("Alpha"));
    assert_eq!(alpha.remote_id, Some(RemoteId::Steam(76561198000000001)));
    assert!(alpha.is_team_0);
    assert_eq!(alpha.boost_amount, 85.0);
    assert_eq!(alpha.last_boost_amount, 80.0);
    assert_eq!(alpha.boost_active, 3);
    assert_eq!(alpha.jump_active, 2);
    assert_eq!(alpha.double_jump_active, 1);
    assert_eq!(alpha.dodge_active, 4);
    assert!(alpha.powerslide_active);
    assert_eq!(alpha.car_body_id, Some(23));
    let input = alpha.input.as_ref().expect("input should convert");
    assert_eq!(input.throttle, 1.0);
    assert_eq!(input.steer, -0.25);
    assert_eq!(input.dodge_forward, 1.0);
    assert_eq!(input.dodge_strafe, -1.0);
    assert!(input.handbrake);
    assert!(!input.jump);
    assert!(input.activate_boost);
    assert!(input.holding_boost);
    let camera = alpha.camera.as_ref().expect("camera should convert");
    assert_eq!(camera.pitch, Some(120));
    assert_eq!(camera.yaw, Some(200));
    assert_eq!(camera.ball_cam_active, Some(true));
    assert_eq!(alpha.dodge_impulse, Some([1.0, 2.0, 3.0]));
    assert_eq!(alpha.dodge_torque, Some([-1.0, 2.5, 0.0]));
    let stats = alpha.match_stats.expect("match stats should convert");
    assert_eq!(
        (
            stats.goals,
            stats.assists,
            stats.saves,
            stats.shots,
            stats.score
        ),
        (1, 2, 3, 4, 350)
    );

    let beta = &live.players[1];
    assert_eq!(beta.name, None);
    assert_eq!(beta.remote_id, Some(RemoteId::Epic("epic-abc-123".to_owned())));
    assert_eq!(beta.input, None);
    assert_eq!(beta.camera, None);
    assert_eq!(beta.dodge_impulse, None);
    assert_eq!(beta.match_stats, None);

    let events = &live.events;
    assert_eq!(events.touches.len(), 1);
    assert_eq!(events.touches[0].timing.frame_and_time, Some((41, 12.4)));
    assert_eq!(events.touches[0].timing.seconds_remaining, Some(210));
    assert_eq!(events.touches[0].player, Some(player_id(0)));
    assert_eq!(events.touches[0].closest_approach_distance, Some(130.5));
    assert_eq!(events.dodge_refreshes.len(), 1);
    assert_eq!(events.dodge_refreshes[0].player, player_id(1));
    assert_eq!(events.dodge_refreshes[0].counter_value, 2);
    assert_eq!(events.boost_pad_events.len(), 1);
    assert_eq!(events.boost_pad_events[0].pad_id, "17");
    assert_eq!(
        events.boost_pad_events[0].kind,
        LiveBoostPadEventKind::PickedUp
    );
    assert_eq!(events.boost_pad_events[0].sequence, 5);
    assert_eq!(events.goals.len(), 1);
    assert!(events.goals[0].scoring_team_is_team_0);
    assert_eq!(events.goals[0].team_zero_score, Some(2));
    assert_eq!(events.player_stat_events.len(), 1);
    assert_eq!(
        events.player_stat_events[0].kind,
        LivePlayerStatEventKind::Shot
    );
    assert!(events.player_stat_events[0].shot_ball.is_some());
    assert!(events.player_stat_events[0].shot_player.is_none());
    assert_eq!(events.demolishes.len(), 1);
    assert_eq!(events.demolishes[0].attacker, player_id(0));
    assert_eq!(events.demolishes[0].victim, player_id(1));
    assert_eq!(events.demolishes[0].active_duration_seconds, 3.0);
}

fn ptr_null() -> *const c_char {
    std::ptr::null()
}

#[test]
fn remote_id_platform_mapping_is_lossless_or_none() {
    let epic_id = cstring("epic-xyz");
    let cases: Vec<(SeRemoteId, Option<RemoteId>)> = vec![
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_NONE, 7, ptr_null()),
            None,
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_STEAM, 76561198000000001, ptr_null()),
            Some(RemoteId::Steam(76561198000000001)),
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_EPIC, 0, epic_id.as_ptr()),
            Some(RemoteId::Epic("epic-xyz".to_owned())),
        ),
        // Epic without an id string has no usable identity.
        (se_remote_id(SE_REMOTE_ID_PLATFORM_EPIC, 9, ptr_null()), None),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_XBOX, 1234, ptr_null()),
            Some(RemoteId::Xbox(1234)),
        ),
        // PsyNet / PlayStation ids carry structured payloads: not lossless.
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_PSYNET, 1234, ptr_null()),
            None,
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_PLAYSTATION, 1234, ptr_null()),
            None,
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_SWITCH, 555, ptr_null()),
            Some(RemoteId::Switch(SwitchId {
                online_id: 555,
                unknown1: Vec::new(),
            })),
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_SPLITSCREEN, 0, ptr_null()),
            Some(RemoteId::SplitScreen(3)),
        ),
        (
            se_remote_id(SE_REMOTE_ID_PLATFORM_QQ, 777, ptr_null()),
            Some(RemoteId::QQ(777)),
        ),
        (se_remote_id(200, 777, ptr_null()), None),
    ];
    for (abi_id, expected) in cases {
        assert_eq!(
            unsafe { remote_id(&abi_id) },
            expected,
            "platform {} should map per the header table",
            abi_id.platform
        );
    }
}

#[test]
fn non_finite_floats_are_sanitized_at_the_abi_boundary() {
    let players = [SePlayerFrame {
        boost_amount: f32::NAN,
        last_boost_amount: f32::INFINITY,
        has_rigid_body: 1,
        rigid_body: SeRigidBody {
            location: SeVec3 {
                x: f32::NAN,
                y: 1.0,
                z: 2.0,
            },
            ..SeRigidBody::default()
        },
        ..SePlayerFrame::default()
    }];
    let touches = [SeTouchEvent {
        closest_approach_distance: f32::NAN,
        has_closest_approach_distance: 1,
        ..SeTouchEvent::default()
    }];
    let frame = SeFrame {
        time: f32::NAN,
        dt: f32::NEG_INFINITY,
        players: players.as_ptr(),
        player_count: players.len(),
        touches: touches.as_ptr(),
        touch_count: touches.len(),
        ..SeFrame::default()
    };
    let live = unsafe { live_frame_from_abi(&frame) }.expect("conversion should succeed");
    assert_eq!(live.time, 0.0);
    assert_eq!(live.dt, 0.0);
    assert_eq!(live.players[0].boost_amount, 0.0);
    assert_eq!(live.players[0].last_boost_amount, 0.0);
    let body = live.players[0].rigid_body.as_ref().unwrap();
    assert_eq!((body.location.x, body.location.y), (0.0, 1.0));
    assert_eq!(live.events.touches[0].closest_approach_distance, None);
    // The wire JSON encoding must accept the sanitized frame.
    serde_json::to_vec(&live).expect("sanitized frame should be JSON-encodable");
}

#[test]
fn malformed_slices_and_null_pointers_are_rejected() {
    let frame = SeFrame {
        players: std::ptr::null(),
        player_count: 2,
        ..SeFrame::default()
    };
    assert!(unsafe { live_frame_from_abi(&frame) }.is_err());

    let mut status = SeStatus::default();
    assert_eq!(
        unsafe { state_export_status(std::ptr::null(), &mut status) },
        -1
    );
    assert_eq!(
        unsafe { state_export_push_frame(std::ptr::null_mut(), &SeFrame::default()) },
        -1
    );
    assert_eq!(
        unsafe { state_export_notify_match_end(std::ptr::null_mut()) },
        -1
    );
    assert_eq!(
        unsafe { state_export_engine_restart(std::ptr::null_mut(), std::ptr::null()) },
        -1
    );
    assert_eq!(
        unsafe { state_export_last_error_len(std::ptr::null()) },
        0
    );
    unsafe { state_export_engine_destroy(std::ptr::null_mut()) };
}

fn expect_next(client: &mut LiveClient) -> ServerMessage {
    client
        .next_message()
        .expect("read should succeed")
        .expect("stream should stay open")
}

/// End-to-end smoke over the whole DLL surface: engine create -> real
/// consumer client -> frames via the FFI -> context update -> match end ->
/// destroy.
#[test]
fn engine_streams_frames_to_a_real_consumer_client() {
    let server_name = cstring("state-export-e2e");
    let config = SeConfig {
        server_name: server_name.as_ptr(),
        ..SeConfig::default()
    };
    let engine = unsafe { state_export_engine_create(&config) };
    assert!(!engine.is_null());

    let mut status = SeStatus::default();
    assert_eq!(unsafe { state_export_status(engine, &mut status) }, 0);
    assert_eq!(status.state, SE_STATE_LISTENING);
    assert_ne!(status.port, 0, "port 0 should resolve to an ephemeral port");
    assert_eq!(unsafe { state_export_last_error_len(engine) }, 0);

    let build_info_len = state_export_build_info_len();
    assert!(build_info_len > 0);
    let mut build_info = vec![0u8; build_info_len];
    let written =
        unsafe { state_export_write_build_info(build_info.as_mut_ptr(), build_info.len()) };
    assert_eq!(written, build_info_len);
    assert!(
        String::from_utf8(build_info)
            .expect("build info should be UTF-8")
            .starts_with("state_export ")
    );

    // Context set before any frame is held and attached to the MatchStart.
    let guid = cstring("D0538C3011F0B32D5C21F3A44E200F5E");
    let map = cstring("Stadium_P");
    let context = SeMatchContext {
        match_guid: guid.as_ptr(),
        map_name: map.as_ptr(),
        playlist_id: 11,
        has_playlist_id: 1,
    };
    assert_eq!(
        unsafe { state_export_set_match_context(engine, &context) },
        0
    );

    let mut client = LiveClient::connect(
        &format!("ws://127.0.0.1:{}", status.port),
        Encoding::Postcard,
        None,
    )
    .expect("client should connect to the engine's server");
    match expect_next(&mut client) {
        ServerMessage::ServerInfo { server, .. } => assert_eq!(server, "state-export-e2e"),
        other => panic!("expected ServerInfo, got {other:?}"),
    }
    match expect_next(&mut client) {
        ServerMessage::EventHistorySnapshot { latest_frame, .. } => {
            assert!(latest_frame.is_none());
        }
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }

    let name0 = cstring("Alpha");
    let name1 = cstring("Beta");
    let players = [
        full_player(
            name0.as_ptr(),
            se_remote_id(SE_REMOTE_ID_PLATFORM_STEAM, 76561198000000001, ptr_null()),
        ),
        SePlayerFrame {
            player_index: 1,
            player_name: name1.as_ptr(),
            is_team_0: 0,
            boost_amount: 33.0,
            last_boost_amount: 33.0,
            ..SePlayerFrame::default()
        },
    ];
    let frame = SeFrame {
        frame_number: 7,
        time: 1.25,
        dt: 1.0 / 120.0,
        seconds_remaining: 280,
        has_seconds_remaining: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 1,
        has_ball: 1,
        ball: se_rigid_body(0.0, 4000.0, 92.75),
        players: players.as_ptr(),
        player_count: players.len(),
        ..SeFrame::default()
    };
    assert_eq!(unsafe { state_export_push_frame(engine, &frame) }, 0);

    match expect_next(&mut client) {
        ServerMessage::MatchStart { meta, .. } => {
            assert_eq!(meta.players.len(), 2);
            assert_eq!(
                meta.players[0].player_id,
                RemoteId::Steam(76561198000000001)
            );
            assert_eq!(meta.players[0].name.as_deref(), Some("Alpha"));
            assert_eq!(
                meta.players[1].player_id,
                RemoteId::SplitScreen(1),
                "players without a platform identity fall back to SplitScreen(player_index)"
            );
            assert_eq!(
                meta.context.match_guid.as_deref(),
                Some("D0538C3011F0B32D5C21F3A44E200F5E")
            );
            assert_eq!(meta.context.playlist_id, Some(11));
            assert_eq!(meta.context.map_name.as_deref(), Some("Stadium_P"));
        }
        other => panic!("expected MatchStart, got {other:?}"),
    }
    match expect_next(&mut client) {
        ServerMessage::Frame { payload, .. } => {
            assert_eq!(payload.frame.frame_number, 7);
            assert_eq!(payload.frame.players.len(), 2);
            let alpha = &payload.frame.players[0];
            assert_eq!(alpha.remote_id, Some(RemoteId::Steam(76561198000000001)));
            let input = alpha.input.as_ref().expect("input should pass through");
            assert_eq!(input.steer, -0.25);
            assert!(input.handbrake);
            let camera = alpha.camera.as_ref().expect("camera should pass through");
            assert_eq!(camera.ball_cam_active, Some(true));
            assert_eq!(alpha.dodge_torque, Some([-1.0, 2.5, 0.0]));
        }
        other => panic!("expected Frame, got {other:?}"),
    }

    // A mid-match context change is re-broadcast as a roster update.
    let updated_context = SeMatchContext {
        playlist_id: 34,
        ..context
    };
    assert_eq!(
        unsafe { state_export_set_match_context(engine, &updated_context) },
        0
    );
    match expect_next(&mut client) {
        ServerMessage::RosterChange { meta, .. } => {
            assert_eq!(meta.context.playlist_id, Some(34));
            assert_eq!(meta.players.len(), 2);
        }
        other => panic!("expected RosterChange, got {other:?}"),
    }

    assert_eq!(unsafe { state_export_notify_match_end(engine) }, 0);
    match expect_next(&mut client) {
        ServerMessage::MatchEnd { .. } => {}
        other => panic!("expected MatchEnd, got {other:?}"),
    }

    assert_eq!(unsafe { state_export_status(engine, &mut status) }, 0);
    assert_eq!(status.client_count, 1);
    assert_eq!(status.frames_sent, 1);
    assert_eq!(status.frames_dropped, 0);

    unsafe { state_export_engine_destroy(engine) };
}

#[test]
fn create_on_an_occupied_port_reports_error_and_restart_recovers() {
    let blocker =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("test listener should bind");
    let occupied_port = blocker.local_addr().unwrap().port();

    let config = SeConfig {
        port: occupied_port,
        ..SeConfig::default()
    };
    let engine = unsafe { state_export_engine_create(&config) };
    assert!(
        !engine.is_null(),
        "create never returns null; failures surface via status/last-error"
    );

    let mut status = SeStatus::default();
    assert_eq!(unsafe { state_export_status(engine, &mut status) }, 0);
    assert_eq!(status.state, SE_STATE_ERROR);
    assert_eq!(status.port, 0);

    let error_len = unsafe { state_export_last_error_len(engine) };
    assert!(error_len > 0);
    let mut error = vec![0u8; error_len];
    let written =
        unsafe { state_export_write_last_error(engine, error.as_mut_ptr(), error.len()) };
    assert_eq!(written, error_len);
    assert!(
        String::from_utf8(error)
            .expect("error should be UTF-8")
            .contains("failed to start export server")
    );

    // Without a running server, push/match-end fail with -2.
    assert_eq!(
        unsafe { state_export_push_frame(engine, &SeFrame::default()) },
        -2
    );
    assert_eq!(unsafe { state_export_notify_match_end(engine) }, -2);

    // Restarting onto an ephemeral port recovers.
    let recover = SeConfig::default();
    assert_eq!(unsafe { state_export_engine_restart(engine, &recover) }, 0);
    assert_eq!(unsafe { state_export_status(engine, &mut status) }, 0);
    assert_eq!(status.state, SE_STATE_LISTENING);
    assert_ne!(status.port, 0);
    assert_eq!(unsafe { state_export_last_error_len(engine) }, 0);

    unsafe { state_export_engine_destroy(engine) };
}
