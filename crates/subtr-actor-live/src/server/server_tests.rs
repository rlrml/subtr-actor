use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;
use crate::model::{LiveDodgeRefreshedEvent, LiveEventTiming, LivePlayerFrame};

const TEST_IO_TIMEOUT: Duration = Duration::from_secs(10);

fn test_config() -> LiveExportServerConfig {
    LiveExportServerConfig {
        // Keep heartbeats out of the way unless a test opts in.
        heartbeat_interval: Duration::from_secs(120),
        server_name: "test-server".to_owned(),
        ..LiveExportServerConfig::default()
    }
}

fn connect(addr: SocketAddr, path: &str) -> WebSocket<TcpStream> {
    let stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(TEST_IO_TIMEOUT)).unwrap();
    stream.set_write_timeout(Some(TEST_IO_TIMEOUT)).unwrap();
    let (ws, _response) =
        tungstenite::client::client(format!("ws://{addr}{path}").as_str(), stream).unwrap();
    ws
}

fn send_hello(ws: &mut WebSocket<TcpStream>, encoding: Encoding, max_frame_hz: Option<f32>) {
    send_hello_versioned(ws, encoding, max_frame_hz, PROTOCOL_MAJOR, PROTOCOL_MINOR);
}

fn send_hello_versioned(
    ws: &mut WebSocket<TcpStream>,
    encoding: Encoding,
    max_frame_hz: Option<f32>,
    protocol_major: u16,
    protocol_minor: u16,
) {
    let hello = ClientMessage::Hello {
        protocol_major,
        protocol_minor,
        encoding,
        max_frame_hz,
    };
    let bytes = hello.encode(Encoding::Json).unwrap();
    ws.send(Message::text(String::from_utf8(bytes).unwrap()))
        .unwrap();
}

fn recv(ws: &mut WebSocket<TcpStream>, encoding: Encoding) -> ServerMessage {
    loop {
        match ws.read().unwrap() {
            Message::Binary(bytes) => {
                assert_eq!(
                    encoding,
                    Encoding::Postcard,
                    "binary frame sent to a json client"
                );
                return ServerMessage::decode(encoding, &bytes).unwrap();
            }
            Message::Text(text) => {
                assert_eq!(
                    encoding,
                    Encoding::Json,
                    "text frame sent to a postcard client"
                );
                return ServerMessage::decode(encoding, text.as_bytes()).unwrap();
            }
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("unexpected websocket message: {other:?}"),
        }
    }
}

fn vec3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn test_rigid_body(location: Vector3f) -> RigidBody {
    RigidBody {
        sleeping: false,
        location,
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(vec3(0.0, 0.0, 0.0)),
        angular_velocity: Some(vec3(0.0, 0.0, 0.0)),
    }
}

fn test_player(index: u32, is_team_0: bool) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index: index,
        name: Some(format!("Player {index}")),
        is_team_0,
        rigid_body: Some(test_rigid_body(vec3(
            index as f32 * 500.0 - 1000.0,
            -4000.0,
            17.0,
        ))),
        boost_amount: 85.0,
        last_boost_amount: 85.0,
        ..LivePlayerFrame::default()
    }
}

/// Ball far away from every player so no touches are synthesized.
fn test_frame(frame_number: u64) -> LiveFrame {
    LiveFrame {
        frame_number,
        time: frame_number as f32 / 30.0,
        dt: 1.0 / 30.0,
        seconds_remaining: Some(280),
        ball_has_been_hit: Some(true),
        live_play: Some(true),
        ball: Some(test_rigid_body(vec3(0.0, 4000.0, 92.75))),
        players: vec![test_player(0, true), test_player(1, false)],
        ..LiveFrame::default()
    }
}

fn frame_with_dodge_refresh(frame_number: u64, counter_value: i32) -> LiveFrame {
    let mut frame = test_frame(frame_number);
    frame.events.dodge_refreshes.push(LiveDodgeRefreshedEvent {
        timing: LiveEventTiming::default(),
        player: RemoteId::SplitScreen(0),
        is_team_0: true,
        counter_value,
    });
    frame
}

fn expect_server_info(message: &ServerMessage) -> u64 {
    match message {
        ServerMessage::ServerInfo {
            protocol_major,
            protocol_minor,
            server,
            seq,
        } => {
            assert_eq!(*protocol_major, PROTOCOL_MAJOR);
            assert_eq!(*protocol_minor, PROTOCOL_MINOR);
            assert_eq!(server, "test-server");
            *seq
        }
        other => panic!("expected ServerInfo, got {other:?}"),
    }
}

#[test]
fn postcard_stream_and_midstream_json_joiner() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let addr = handle.local_addr();

    // (a) A postcard client subscribed from the start.
    let mut client_a = connect(addr, "/");
    send_hello(&mut client_a, Encoding::Postcard, None);
    expect_server_info(&recv(&mut client_a, Encoding::Postcard));
    match recv(&mut client_a, Encoding::Postcard) {
        ServerMessage::EventHistorySnapshot {
            history,
            latest_frame,
            ..
        } => {
            assert_eq!(history, WireEventHistory::default());
            assert!(latest_frame.is_none());
        }
        other => panic!("expected empty snapshot, got {other:?}"),
    }

    for frame_number in 0..5 {
        handle.push_frame(frame_with_dodge_refresh(
            frame_number,
            frame_number as i32 + 1,
        ));
    }

    match recv(&mut client_a, Encoding::Postcard) {
        ServerMessage::MatchStart { meta, .. } => assert_eq!(meta.players.len(), 2),
        other => panic!("expected MatchStart, got {other:?}"),
    }
    let mut accumulated = WireEventHistory::default();
    let mut last_payload = None;
    for _ in 0..5 {
        match recv(&mut client_a, Encoding::Postcard) {
            ServerMessage::Frame { payload, .. } => {
                accumulated.append_frame_events(&payload.derived_events);
                last_payload = Some(payload);
            }
            other => panic!("expected Frame, got {other:?}"),
        }
    }
    assert_eq!(accumulated.dodge_refreshed_events.len(), 5);

    // (b) A json client joining mid-stream gets a consistent prologue with
    // contiguous seq and the same accumulated history as the early joiner.
    let mut client_b = connect(addr, "/");
    send_hello(&mut client_b, Encoding::Json, None);
    let info_seq = expect_server_info(&recv(&mut client_b, Encoding::Json));
    let match_start_seq = match recv(&mut client_b, Encoding::Json) {
        ServerMessage::MatchStart { seq, meta } => {
            assert_eq!(meta.players.len(), 2);
            seq
        }
        other => panic!("expected MatchStart, got {other:?}"),
    };
    assert_eq!(match_start_seq, info_seq + 1);
    let snapshot_seq = match recv(&mut client_b, Encoding::Json) {
        ServerMessage::EventHistorySnapshot {
            seq,
            history,
            latest_frame,
        } => {
            assert_eq!(history, accumulated);
            assert_eq!(latest_frame.as_deref(), last_payload.as_deref());
            seq
        }
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    };
    assert_eq!(snapshot_seq, match_start_seq + 1);

    // The next frame reaches both clients, contiguous for the new joiner.
    handle.push_frame(frame_with_dodge_refresh(5, 6));
    let payload_b = match recv(&mut client_b, Encoding::Json) {
        ServerMessage::Frame { seq, payload } => {
            assert_eq!(seq, snapshot_seq + 1);
            payload
        }
        other => panic!("expected Frame, got {other:?}"),
    };
    let payload_a = match recv(&mut client_a, Encoding::Postcard) {
        ServerMessage::Frame { payload, .. } => payload,
        other => panic!("expected Frame, got {other:?}"),
    };
    assert_eq!(payload_a, payload_b);

    handle.shutdown();
}

// (c) `?format=json` subscribes without any Hello.
#[test]
fn query_param_json_subscription_without_hello() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/?format=json");
    expect_server_info(&recv(&mut client, Encoding::Json));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }
    assert_eq!(handle.client_count(), 1);
    handle.shutdown();
}

// (d) A protocol_major mismatch is rejected with a descriptive close reason.
#[test]
fn protocol_major_mismatch_is_rejected_with_close_reason() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/");
    send_hello_versioned(
        &mut client,
        Encoding::Json,
        None,
        PROTOCOL_MAJOR + 1,
        PROTOCOL_MINOR,
    );
    let close = loop {
        match client.read() {
            Ok(Message::Close(frame)) => break frame,
            Ok(_) => continue,
            Err(error) => panic!("expected a close frame, got error {error:?}"),
        }
    };
    let close = close.expect("close frame should carry a reason");
    assert!(
        close.reason.as_str().contains("protocol"),
        "reason should mention the protocol: {:?}",
        close.reason
    );
    handle.shutdown();
}

// A postcard client with a minor-version mismatch is also rejected.
#[test]
fn postcard_minor_mismatch_is_rejected() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/");
    send_hello_versioned(
        &mut client,
        Encoding::Postcard,
        None,
        PROTOCOL_MAJOR,
        PROTOCOL_MINOR + 1,
    );
    loop {
        match client.read() {
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(error) => panic!("expected a close frame, got error {error:?}"),
        }
    }
    handle.shutdown();
}

// (e) shutdown() joins cleanly while clients are connected.
#[test]
fn shutdown_joins_cleanly_with_connected_clients() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let addr = handle.local_addr();
    let mut client_a = connect(addr, "/");
    send_hello(&mut client_a, Encoding::Postcard, None);
    let mut client_b = connect(addr, "/?format=json");
    expect_server_info(&recv(&mut client_a, Encoding::Postcard));
    expect_server_info(&recv(&mut client_b, Encoding::Json));
    handle.push_frame(test_frame(0));

    let started = Instant::now();
    handle.shutdown();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "shutdown took {:?}",
        started.elapsed()
    );
    assert_eq!(handle.client_count(), 0);

    // The clients observe the server-initiated close (or a dropped socket).
    loop {
        match client_a.read() {
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        }
    }
}

// (f) Ingest overflow drops raw frames but coalesces their explicit events
// into surviving frames, so no hook-driven event is lost.
#[test]
fn ingest_overflow_coalesces_explicit_events() {
    let handle = LiveExportServer::start(LiveExportServerConfig {
        max_ingest_frames: 1,
        // Large enough that the test client itself is never disconnected for
        // queue overflow while we stuff the ingest side.
        max_client_queue: 1_000_000,
        ..test_config()
    })
    .unwrap();
    let mut client = connect(handle.local_addr(), "/");
    send_hello(&mut client, Encoding::Postcard, None);
    expect_server_info(&recv(&mut client, Encoding::Postcard));
    match recv(&mut client, Encoding::Postcard) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }

    // Push bursts until the drop counter moves; every frame carries a unique
    // dodge-refresh event.
    let mut pushed: i32 = 0;
    let mut batches = 0;
    while handle.stats().frames_dropped == 0 {
        batches += 1;
        assert!(
            batches <= 200,
            "server kept up with every burst; cannot exercise overflow"
        );
        for _ in 0..500 {
            pushed += 1;
            handle.push_frame(frame_with_dodge_refresh(pushed as u64, pushed));
        }
    }

    let expected = pushed as usize;
    let mut received = 0usize;
    let deadline = Instant::now() + Duration::from_secs(30);
    while received < expected {
        assert!(
            Instant::now() < deadline,
            "timed out with {received} of {expected} dodge-refresh events"
        );
        if let ServerMessage::Frame { payload, .. } = recv(&mut client, Encoding::Postcard) {
            received += payload.derived_events.dodge_refreshed_events.len();
        }
    }
    assert_eq!(received, expected);
    let stats = handle.stats();
    assert!(stats.frames_dropped > 0);
    assert!(stats.frames_sent < pushed as u64);
    handle.shutdown();
}

// Heartbeats flow when no frames do.
#[test]
fn heartbeat_when_idle() {
    let handle = LiveExportServer::start(LiveExportServerConfig {
        heartbeat_interval: Duration::from_millis(100),
        server_name: "test-server".to_owned(),
        ..LiveExportServerConfig::default()
    })
    .unwrap();
    let mut client = connect(handle.local_addr(), "/?format=json");
    expect_server_info(&recv(&mut client, Encoding::Json));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }
    match recv(&mut client, Encoding::Json) {
        ServerMessage::Heartbeat { time, .. } => assert!(time > 0.0),
        other => panic!("expected Heartbeat, got {other:?}"),
    }
    handle.shutdown();
}

// max_frame_hz downsamples event-free frames but never event-carrying ones.
#[test]
fn max_frame_hz_downsamples_only_event_free_frames() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/");
    send_hello(&mut client, Encoding::Json, Some(1.0));
    expect_server_info(&recv(&mut client, Encoding::Json));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }

    // A burst of event-free frames all lands within the 1s budget: exactly one
    // Frame passes the cap. The event-carrying frame afterwards must be
    // delivered regardless of the cap.
    for frame_number in 0..10 {
        handle.push_frame(test_frame(frame_number));
    }
    handle.push_frame(frame_with_dodge_refresh(10, 1));

    match recv(&mut client, Encoding::Json) {
        ServerMessage::MatchStart { .. } => {}
        other => panic!("expected MatchStart, got {other:?}"),
    }
    match recv(&mut client, Encoding::Json) {
        ServerMessage::Frame { payload, .. } => {
            assert!(!payload.derived_events.has_discrete_events());
        }
        other => panic!("expected the first Frame, got {other:?}"),
    }
    match recv(&mut client, Encoding::Json) {
        ServerMessage::Frame { payload, .. } => {
            assert_eq!(payload.derived_events.dodge_refreshed_events.len(), 1);
        }
        other => panic!("expected the event-carrying Frame, got {other:?}"),
    }
    handle.shutdown();
}

// set_match_context re-broadcasts the roster as a RosterChange carrying the
// context when a match is live, attaches held context to the eventual
// MatchStart, and match_end clears it.
#[test]
fn match_context_flows_through_meta() {
    let context = LiveMatchContext {
        match_guid: Some("A0538C3011F0B32D5C21F3A44E200F5E".to_owned()),
        playlist_id: Some(10),
        map_name: Some("Stadium_P".to_owned()),
    };
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/?format=json");
    expect_server_info(&recv(&mut client, Encoding::Json));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }

    // Context set before any frame is held and attached to the MatchStart.
    handle.set_match_context(context.clone());
    handle.push_frame(test_frame(0));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::MatchStart { meta, .. } => {
            assert_eq!(meta.context, context);
            assert_eq!(meta.players.len(), 2);
        }
        other => panic!("expected MatchStart, got {other:?}"),
    }
    match recv(&mut client, Encoding::Json) {
        ServerMessage::Frame { .. } => {}
        other => panic!("expected Frame, got {other:?}"),
    }

    // A mid-match context change re-broadcasts the roster as a RosterChange.
    let updated = LiveMatchContext {
        playlist_id: Some(11),
        ..context.clone()
    };
    handle.set_match_context(updated.clone());
    match recv(&mut client, Encoding::Json) {
        ServerMessage::RosterChange { meta, .. } => {
            assert_eq!(meta.context, updated);
            assert_eq!(meta.players.len(), 2);
        }
        other => panic!("expected RosterChange, got {other:?}"),
    }

    // match_end clears the context: the next match starts without it.
    handle.match_end();
    match recv(&mut client, Encoding::Json) {
        ServerMessage::MatchEnd { .. } => {}
        other => panic!("expected MatchEnd, got {other:?}"),
    }
    handle.push_frame(test_frame(1));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::MatchStart { meta, .. } => {
            assert_eq!(meta.context, LiveMatchContext::default());
        }
        other => panic!("expected MatchStart, got {other:?}"),
    }
    handle.shutdown();
}

// match_end broadcasts MatchEnd, resets state, and the next frame starts a
// fresh match with a new MatchStart.
#[test]
fn match_end_resets_and_restarts() {
    let handle = LiveExportServer::start(test_config()).unwrap();
    let mut client = connect(handle.local_addr(), "/?format=json");
    expect_server_info(&recv(&mut client, Encoding::Json));
    match recv(&mut client, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { .. } => {}
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }

    handle.push_frame(frame_with_dodge_refresh(0, 1));
    handle.match_end();
    handle.push_frame(frame_with_dodge_refresh(1, 1));

    let mut kinds = Vec::new();
    while kinds.len() < 5 {
        kinds.push(match recv(&mut client, Encoding::Json) {
            ServerMessage::MatchStart { .. } => "match_start",
            ServerMessage::Frame { .. } => "frame",
            ServerMessage::MatchEnd { .. } => "match_end",
            other => panic!("unexpected message {other:?}"),
        });
    }
    assert_eq!(
        kinds,
        ["match_start", "frame", "match_end", "match_start", "frame"]
    );

    // A fresh joiner after the second frame sees only the new match's history
    // (one dodge refresh, not two).
    let mut late = connect(handle.local_addr(), "/?format=json");
    expect_server_info(&recv(&mut late, Encoding::Json));
    match recv(&mut late, Encoding::Json) {
        ServerMessage::MatchStart { .. } => {}
        other => panic!("expected MatchStart, got {other:?}"),
    }
    match recv(&mut late, Encoding::Json) {
        ServerMessage::EventHistorySnapshot { history, .. } => {
            assert_eq!(history.dodge_refreshed_events.len(), 1);
        }
        other => panic!("expected EventHistorySnapshot, got {other:?}"),
    }
    handle.shutdown();
}
