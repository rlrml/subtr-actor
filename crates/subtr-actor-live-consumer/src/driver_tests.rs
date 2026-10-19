use super::*;
use subtr_actor_live::{FramePayload, LiveFrame, LivePlayerFrame};

fn frame(frame_number: u64) -> LiveFrame {
    LiveFrame {
        frame_number,
        time: frame_number as f32 / 30.0,
        dt: 1.0 / 30.0,
        players: vec![LivePlayerFrame {
            player_index: 0,
            name: Some("Player 0".to_owned()),
            is_team_0: true,
            ..LivePlayerFrame::default()
        }],
        ..LiveFrame::default()
    }
}

fn frame_message(seq: u64, frame_number: u64) -> ServerMessage {
    ServerMessage::Frame {
        seq,
        payload: Box::new(FramePayload {
            frame: frame(frame_number),
            derived_events: Default::default(),
            live_play: Default::default(),
        }),
    }
}

fn server_info(seq: u64) -> ServerMessage {
    ServerMessage::ServerInfo {
        protocol_major: subtr_actor_live::PROTOCOL_MAJOR,
        protocol_minor: subtr_actor_live::PROTOCOL_MINOR,
        server: "test".to_owned(),
        seq,
    }
}

#[test]
fn downsampled_frames_integrate_dt_over_the_gap() {
    assert_eq!(effective_dt(None, &frame(10)), 1.0 / 30.0);
    assert_eq!(effective_dt(Some((9, 9.0 / 30.0)), &frame(10)), 1.0 / 30.0);
    let gap_dt = effective_dt(Some((4, 4.0 / 30.0)), &frame(10));
    assert!((gap_dt - 6.0 / 30.0).abs() < 1e-6, "{gap_dt}");
}

#[test]
fn reconnect_finishes_and_resets_previous_session() {
    let mut store = LiveStateStore::new();
    let mut driver = LiveGraphDriver::new();
    let mut outputs = Vec::new();
    let mut sink = |output| outputs.push(output);

    driver
        .on_message(&mut store, server_info(1), &mut sink)
        .unwrap();
    for (seq, frame_number) in (2..20).zip(0..) {
        driver
            .on_message(&mut store, frame_message(seq, frame_number), &mut sink)
            .unwrap();
    }
    assert!(store.latest().is_some());

    // The server dropped us and we resubscribed; seq keeps increasing.
    driver
        .on_message(&mut store, server_info(40), &mut sink)
        .unwrap();

    assert!(matches!(outputs.last(), Some(DriverOutput::Resubscribed)));
    assert!(store.latest().is_none());
    assert!(store.meta().is_none());
}
