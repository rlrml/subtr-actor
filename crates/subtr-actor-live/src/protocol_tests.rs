use boxcars::{Ps4Id, Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{GameplayPhase, LivePlayState};

use super::*;
use crate::model::{
    LiveBoostPadEvent, LiveBoostPadEventKind, LiveCameraState, LiveControllerInput,
    LiveDemolishEvent, LiveDodgeRefreshedEvent, LiveEventTiming, LiveExplicitEvents, LiveFrame,
    LiveGoalEvent, LiveMatchStats, LivePlayerFrame, LivePlayerStatEvent, LivePlayerStatEventKind,
    LiveTouchEvent,
};
use crate::wire::{
    WireBoostPadEvent, WireBoostPadEventKind, WireDemoEventSample, WireDemolishInfo,
    WireDodgeRefreshedEvent, WireEventHistory, WireFrameEventsState, WireGoalEvent,
    WirePlayerStatEvent, WirePlayerStatEventKind, WireShotEventMetadata, WireShotGoalLineCrossing,
    WireShotGoalLineCrossingPredictionKind, WireShotGoalLineCrossingUnavailableReason,
    WireShotGoalTargetHit, WireShotGoalTargetHitKind, WireShotSaveMetadata, WireTouchEvent,
};

fn vec3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn rigid_body() -> RigidBody {
    RigidBody {
        sleeping: false,
        location: vec3(101.5, -2044.25, 92.75),
        rotation: Quaternion {
            x: 0.1,
            y: -0.2,
            z: 0.3,
            w: 0.9,
        },
        linear_velocity: Some(vec3(500.0, -250.0, 12.5)),
        angular_velocity: Some(vec3(0.5, 1.5, -2.5)),
    }
}

fn remote_id(index: u32) -> RemoteId {
    match index % 3 {
        0 => RemoteId::Steam(76_561_198_122_624_102 + u64::from(index)),
        1 => RemoteId::Epic(format!("epic-player-{index}")),
        _ => RemoteId::PlayStation(Ps4Id {
            online_id: 9_000_000 + u64::from(index),
            name: format!("psn-{index}"),
            unknown1: vec![1, 2, 3],
        }),
    }
}

fn full_player(index: u32) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index: index,
        name: Some(format!("Player {index}")),
        remote_id: Some(remote_id(index)),
        is_team_0: index.is_multiple_of(2),
        rigid_body: Some(rigid_body()),
        boost_amount: 128.0,
        last_boost_amount: 140.0,
        boost_active: 3,
        jump_active: 1,
        double_jump_active: 2,
        dodge_active: 5,
        powerslide_active: true,
        input: Some(LiveControllerInput {
            throttle: 1.0,
            steer: -0.5,
            pitch: 0.25,
            yaw: -0.75,
            roll: 0.125,
            dodge_forward: 1.0,
            dodge_strafe: -1.0,
            handbrake: true,
            jump: true,
            activate_boost: true,
            holding_boost: true,
        }),
        camera: Some(LiveCameraState {
            pitch: Some(120),
            yaw: Some(200),
            ball_cam_active: Some(true),
        }),
        dodge_impulse: Some([1.0, -2.0, 0.5]),
        dodge_torque: Some([-1.8, 1.8, 0.0]),
        car_body_id: Some(23),
        match_stats: Some(LiveMatchStats {
            goals: 2,
            assists: 1,
            saves: 3,
            shots: 5,
            score: 512,
        }),
    }
}

fn timing() -> LiveEventTiming {
    LiveEventTiming {
        frame_and_time: Some((1200, 41.5)),
        seconds_remaining: Some(180),
    }
}

fn full_explicit_events() -> LiveExplicitEvents {
    LiveExplicitEvents {
        touches: vec![LiveTouchEvent {
            timing: timing(),
            player: Some(remote_id(0)),
            is_team_0: true,
            closest_approach_distance: Some(4.5),
        }],
        dodge_refreshes: vec![LiveDodgeRefreshedEvent {
            timing: timing(),
            player: remote_id(1),
            is_team_0: false,
            counter_value: 7,
        }],
        boost_pad_events: vec![LiveBoostPadEvent {
            timing: timing(),
            pad_id: "big_pad_3".to_owned(),
            kind: LiveBoostPadEventKind::PickedUp,
            sequence: 9,
            player: Some(remote_id(2)),
        }],
        goals: vec![LiveGoalEvent {
            timing: timing(),
            scoring_team_is_team_0: true,
            player: Some(remote_id(0)),
            team_zero_score: Some(2),
            team_one_score: Some(1),
        }],
        player_stat_events: vec![LivePlayerStatEvent {
            timing: timing(),
            player: remote_id(3),
            is_team_0: false,
            kind: LivePlayerStatEventKind::Shot,
            shot_ball: Some(rigid_body()),
            shot_player: Some(rigid_body()),
        }],
        demolishes: vec![LiveDemolishEvent {
            timing: timing(),
            attacker: remote_id(4),
            victim: remote_id(5),
            attacker_velocity: vec3(2200.0, 0.0, 0.0),
            victim_velocity: vec3(-300.0, 100.0, 0.0),
            victim_location: vec3(0.0, 4800.0, 17.0),
            active_duration_seconds: 3.0,
        }],
    }
}

fn full_live_frame() -> LiveFrame {
    LiveFrame {
        frame_number: 1200,
        time: 41.5,
        dt: 1.0 / 120.0,
        seconds_remaining: Some(180),
        game_state: Some(1),
        kickoff_countdown_time: Some(0),
        ball_has_been_hit: Some(true),
        team_zero_score: Some(2),
        team_one_score: Some(1),
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: Some(false),
        live_play: Some(true),
        ball: Some(rigid_body()),
        players: (0..6).map(full_player).collect(),
        events: full_explicit_events(),
    }
}

fn sparse_live_frame() -> LiveFrame {
    LiveFrame {
        frame_number: 3,
        time: 0.1,
        dt: 1.0 / 30.0,
        players: vec![LivePlayerFrame {
            player_index: 0,
            is_team_0: true,
            ..LivePlayerFrame::default()
        }],
        ..LiveFrame::default()
    }
}

fn full_shot_metadata() -> WireShotEventMetadata {
    WireShotEventMetadata {
        shot_touch_position: vec3(0.0, 3000.0, 100.0),
        ball_position: vec3(1.0, 3010.0, 110.0),
        ball_velocity: Some(vec3(0.0, 2500.0, 300.0)),
        ball_speed: Some(2518.0),
        player_position: Some(vec3(0.0, 2900.0, 17.0)),
        player_velocity: Some(vec3(0.0, 1400.0, 0.0)),
        player_speed: Some(1400.0),
        player_distance_to_ball: Some(140.0),
        target_goal_position: vec3(0.0, 5120.0, 110.0),
        distance_to_goal_center: 2110.0,
        distance_to_goal_line: 2110.0,
        ball_goal_alignment: Some(0.98),
        ball_speed_toward_goal: Some(2450.0),
        projected_goal_line_crossing: Some(WireShotGoalLineCrossing {
            time_after_shot: 0.86,
            prediction_start_time: Some(41.5),
            prediction_start_frame: Some(1200),
            position: vec3(120.0, 5120.0, 300.0),
            velocity: Some(vec3(0.0, 2400.0, 100.0)),
            inside_goal_mouth: true,
            prediction_kind: WireShotGoalLineCrossingPredictionKind::SurfaceBounces,
        }),
        projected_goal_line_crossing_unavailable_reason: Some(
            WireShotGoalLineCrossingUnavailableReason::NoUsableProjection,
        ),
        projected_goal_target_hit: Some(WireShotGoalTargetHit {
            time_after_shot: 0.9,
            prediction_start_time: Some(41.5),
            prediction_start_frame: Some(1200),
            position: vec3(120.0, 5120.0, 300.0),
            velocity: Some(vec3(0.0, 2400.0, 100.0)),
            hit_kind: WireShotGoalTargetHitKind::GoalLine,
        }),
        resulting_save: Some(WireShotSaveMetadata {
            time: 42.4,
            frame: 1308,
            player: remote_id(1),
            player_position: Some(vec3(20.0, 5000.0, 90.0)),
            is_team_0: false,
        }),
    }
}

fn full_wire_events() -> WireFrameEventsState {
    WireFrameEventsState {
        active_demos: vec![WireDemoEventSample {
            attacker: remote_id(4),
            victim: remote_id(5),
        }],
        demo_events: vec![WireDemolishInfo {
            time: 41.5,
            seconds_remaining: 180,
            frame: 1200,
            attacker: remote_id(4),
            victim: remote_id(5),
            attacker_velocity: vec3(2200.0, 0.0, 0.0),
            victim_velocity: vec3(-300.0, 100.0, 0.0),
            attacker_location: Some(vec3(0.0, 4700.0, 17.0)),
            victim_location: vec3(0.0, 4800.0, 17.0),
        }],
        boost_pad_events: vec![WireBoostPadEvent {
            time: 41.5,
            frame: 1200,
            pad_id: "big_pad_3".to_owned(),
            player: Some(remote_id(2)),
            player_position: Some(vec3(-3072.0, 4096.0, 73.0)),
            kind: WireBoostPadEventKind::PickedUp { sequence: 9 },
        }],
        touch_events: vec![WireTouchEvent {
            touch_id: Some(41),
            time: 41.5,
            frame: 1200,
            team_is_team_0: true,
            player: Some(remote_id(0)),
            player_position: Some(vec3(101.5, -2044.25, 92.75)),
            closest_approach_distance: Some(0.0),
            contact_local_ball_position: Some([10.0, 20.0, 30.0]),
            contact_local_hitbox_point: Some([1.0, 2.0, 3.0]),
            contact_world_hitbox_point: Some([100.0, -2000.0, 95.0]),
            dodge_contact: true,
        }],
        dodge_refreshed_counter_available: true,
        dodge_refreshed_events: vec![WireDodgeRefreshedEvent {
            time: 41.5,
            frame: 1200,
            player: remote_id(1),
            player_position: Some([5.0, 6.0, 7.0]),
            is_team_0: false,
            counter_value: 7,
        }],
        player_stat_events: vec![WirePlayerStatEvent {
            time: 41.5,
            frame: 1200,
            player: remote_id(3),
            player_position: Some(vec3(0.0, 2900.0, 17.0)),
            is_team_0: false,
            kind: WirePlayerStatEventKind::Shot,
            shot: Some(full_shot_metadata()),
        }],
        goal_events: vec![WireGoalEvent {
            time: 41.5,
            frame: 1200,
            scoring_team_is_team_0: true,
            player: Some(remote_id(0)),
            player_position: Some(vec3(0.0, 5000.0, 100.0)),
            team_zero_score: Some(2),
            team_one_score: Some(1),
        }],
    }
}

fn full_history() -> WireEventHistory {
    let mut history = WireEventHistory::default();
    history.append_frame_events(&full_wire_events());
    history.append_frame_events(&full_wire_events());
    history
}

fn full_frame_payload() -> FramePayload {
    FramePayload {
        frame: full_live_frame(),
        derived_events: full_wire_events(),
        live_play: LivePlayState::new(GameplayPhase::ActivePlay),
    }
}

fn sparse_frame_payload() -> FramePayload {
    FramePayload {
        frame: sparse_live_frame(),
        derived_events: WireFrameEventsState::default(),
        live_play: LivePlayState::default(),
    }
}

fn all_server_messages() -> Vec<ServerMessage> {
    let mut full_meta = LiveMatchMeta::from_player_frames(&full_live_frame().players);
    full_meta.context = crate::meta::LiveMatchContext {
        match_guid: Some("D0538C3011F0B32D5C21F3A44E200F5E".to_owned()),
        playlist_id: Some(11),
        map_name: Some("Stadium_P".to_owned()),
    };
    let sparse_meta = LiveMatchMeta::from_player_frames(&sparse_live_frame().players);
    vec![
        ServerMessage::ServerInfo {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            server: "test-server".to_owned(),
            seq: 1,
        },
        ServerMessage::MatchStart {
            seq: 2,
            meta: full_meta.clone(),
        },
        ServerMessage::MatchStart {
            seq: 3,
            meta: sparse_meta,
        },
        ServerMessage::RosterChange {
            seq: 4,
            meta: full_meta,
        },
        ServerMessage::EventHistorySnapshot {
            seq: 5,
            history: full_history(),
            latest_frame: Some(Box::new(full_frame_payload())),
        },
        ServerMessage::EventHistorySnapshot {
            seq: 6,
            history: WireEventHistory::default(),
            latest_frame: None,
        },
        ServerMessage::Frame {
            seq: 7,
            payload: Box::new(full_frame_payload()),
        },
        ServerMessage::Frame {
            seq: 8,
            payload: Box::new(sparse_frame_payload()),
        },
        ServerMessage::MatchEnd { seq: 9 },
        ServerMessage::Heartbeat {
            seq: 10,
            time: 1_752_000_000.25,
        },
    ]
}

/// Round-trip equality through postcard is the guard against a wire type
/// regressing to `skip_serializing_if` (postcard would desynchronize) or
/// losing `Deserialize`.
#[test]
fn server_messages_round_trip_in_both_encodings() {
    for message in all_server_messages() {
        for encoding in [Encoding::Postcard, Encoding::Json] {
            let bytes = message.encode(encoding).unwrap();
            let decoded = ServerMessage::decode(encoding, &bytes).unwrap();
            assert_eq!(decoded, message, "{encoding:?} round trip");
        }
    }
}

#[test]
fn client_messages_round_trip_in_both_encodings() {
    let messages = [
        ClientMessage::Hello {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            encoding: Encoding::Postcard,
            max_frame_hz: Some(30.0),
        },
        ClientMessage::Hello {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            encoding: Encoding::Json,
            max_frame_hz: None,
        },
    ];
    for message in messages {
        for encoding in [Encoding::Postcard, Encoding::Json] {
            let bytes = message.encode(encoding).unwrap();
            let decoded = ClientMessage::decode(encoding, &bytes).unwrap();
            assert_eq!(decoded, message, "{encoding:?} round trip");
        }
        let json = String::from_utf8(message.encode(Encoding::Json).unwrap()).unwrap();
        assert_eq!(ClientMessage::decode_json(&json).unwrap(), message);
    }
}

#[test]
fn seq_accessor_matches_every_variant() {
    for (index, message) in all_server_messages().into_iter().enumerate() {
        assert_eq!(message.seq(), index as u64 + 1);
    }
}

#[test]
fn six_player_postcard_frame_stays_small() {
    let message = ServerMessage::Frame {
        seq: 1,
        payload: Box::new(full_frame_payload()),
    };
    let bytes = message.encode(Encoding::Postcard).unwrap();
    assert!(
        bytes.len() < 4096,
        "fully-populated 6-player postcard frame grew to {} bytes",
        bytes.len()
    );
}

#[test]
fn decode_rejects_garbage() {
    assert!(ServerMessage::decode(Encoding::Postcard, &[0xff, 0xff, 0xff]).is_err());
    assert!(ServerMessage::decode(Encoding::Json, b"{not json").is_err());
    assert!(ClientMessage::decode_json("{}").is_err());
}

#[test]
fn version_rule_major_must_match_and_postcard_minor_is_exact() {
    assert!(protocol_versions_compatible(
        Encoding::Postcard,
        PROTOCOL_MAJOR,
        PROTOCOL_MINOR
    ));
    assert!(protocol_versions_compatible(
        Encoding::Json,
        PROTOCOL_MAJOR,
        PROTOCOL_MINOR
    ));
    // JSON tolerates minor drift; postcard does not.
    assert!(protocol_versions_compatible(
        Encoding::Json,
        PROTOCOL_MAJOR,
        PROTOCOL_MINOR + 1
    ));
    assert!(!protocol_versions_compatible(
        Encoding::Postcard,
        PROTOCOL_MAJOR,
        PROTOCOL_MINOR + 1
    ));
    // Major must match exactly for both encodings.
    assert!(!protocol_versions_compatible(
        Encoding::Json,
        PROTOCOL_MAJOR + 1,
        PROTOCOL_MINOR
    ));
    assert!(!protocol_versions_compatible(
        Encoding::Postcard,
        PROTOCOL_MAJOR + 1,
        PROTOCOL_MINOR
    ));
}

/// The wire mirrors must convert losslessly in both directions.
#[test]
fn wire_frame_events_convert_both_ways() {
    let wire = full_wire_events();
    let core: subtr_actor::FrameEventsState = wire.clone().into();
    let back: WireFrameEventsState = core.into();
    assert_eq!(back, wire);

    let history = full_history();
    let core_history: crate::generator::LiveEventHistory = history.clone().into();
    let back_history: WireEventHistory = core_history.into();
    assert_eq!(back_history, history);
}
