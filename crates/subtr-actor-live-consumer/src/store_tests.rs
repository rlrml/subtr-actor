use super::*;
use subtr_actor_live::{
    FramePayload, LiveFrame, LivePlayerFrame, LiveRosterPlayer, WireDodgeRefreshedEvent,
    WireEventHistory, WireFrameEventsState,
};

fn dodge_event(frame: usize) -> WireDodgeRefreshedEvent {
    WireDodgeRefreshedEvent {
        time: frame as f32 / 30.0,
        frame,
        player: boxcars::RemoteId::SplitScreen(0),
        player_position: None,
        is_team_0: true,
        counter_value: frame as i32,
    }
}

fn frame_message(seq: u64, frame_number: u64, events: WireFrameEventsState) -> ServerMessage {
    ServerMessage::Frame {
        seq,
        payload: Box::new(FramePayload {
            frame: LiveFrame {
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
            },
            derived_events: events,
            live_play: LivePlayState::default(),
        }),
    }
}

fn events_with_dodge(frame: usize) -> WireFrameEventsState {
    WireFrameEventsState {
        dodge_refreshed_events: vec![dodge_event(frame)],
        ..WireFrameEventsState::default()
    }
}

fn test_meta() -> LiveMatchMeta {
    LiveMatchMeta {
        players: vec![LiveRosterPlayer {
            player_id: boxcars::RemoteId::SplitScreen(0),
            name: Some("Player 0".to_owned()),
            is_team_0: true,
            car_body_id: None,
        }],
        ..LiveMatchMeta::default()
    }
}

#[test]
fn frames_append_to_history_and_snapshot_replaces_it() {
    let mut store = LiveStateStore::new();
    assert_eq!(
        store.apply(frame_message(1, 0, events_with_dodge(0))),
        Applied::Frame
    );
    assert_eq!(
        store.apply(frame_message(2, 1, events_with_dodge(1))),
        Applied::Frame
    );
    assert_eq!(store.history().dodge_refreshed_events.len(), 2);
    assert_eq!(store.latest().unwrap().frame.frame_number, 1);

    // A snapshot replaces the accumulated history wholesale.
    let mut snapshot_history = WireEventHistory::default();
    snapshot_history.dodge_refreshed_events.push(dodge_event(7));
    let applied = store.apply(ServerMessage::EventHistorySnapshot {
        seq: 3,
        history: snapshot_history,
        latest_frame: None,
    });
    assert_eq!(applied, Applied::Snapshot);
    assert_eq!(store.history().dodge_refreshed_events.len(), 1);
    assert_eq!(store.history().dodge_refreshed_events[0].frame, 7);
    // The pre-snapshot latest frame is retained when the snapshot has none.
    assert_eq!(store.latest().unwrap().frame.frame_number, 1);
}

#[test]
fn view_requires_a_frame_and_exposes_history() {
    let mut store = LiveStateStore::new();
    assert!(store.view().is_none());
    store.apply(ServerMessage::MatchStart {
        seq: 1,
        meta: test_meta(),
    });
    assert!(store.view().is_none());
    store.apply(frame_message(2, 0, events_with_dodge(0)));
    let view = store.view().expect("frame applied");
    use subtr_actor::ProcessorView;
    assert_eq!(view.player_count(), 1);
    assert_eq!(view.dodge_refreshed_events().len(), 1);
    assert!(view.get_replay_meta().is_ok());
}

#[test]
fn match_end_clears_match_state_but_not_seq_tracking() {
    let mut store = LiveStateStore::new();
    store.apply(ServerMessage::MatchStart {
        seq: 5,
        meta: test_meta(),
    });
    store.apply(frame_message(6, 0, events_with_dodge(0)));
    assert_eq!(
        store.apply(ServerMessage::MatchEnd { seq: 7 }),
        Applied::MatchEnd
    );
    assert!(store.meta().is_none());
    assert!(store.latest().is_none());
    assert!(store.history().dodge_refreshed_events.is_empty());

    // Seq tracking survives the match reset: a later, higher seq is normal...
    assert_eq!(
        store.apply(ServerMessage::Heartbeat { seq: 8, time: 1.0 }),
        Applied::Heartbeat
    );
}

#[test]
fn seq_decrease_reports_server_restart_and_resets() {
    let mut store = LiveStateStore::new();
    store.apply(ServerMessage::MatchStart {
        seq: 100,
        meta: test_meta(),
    });
    store.apply(frame_message(101, 0, events_with_dodge(0)));
    assert!(store.meta().is_some());

    // A fresh server restarts its seq counter near zero; the store resets
    // and still applies the message (here: the reconnect prologue's info).
    let applied = store.apply(ServerMessage::ServerInfo {
        protocol_major: 1,
        protocol_minor: 0,
        server: "restarted".to_owned(),
        seq: 1,
    });
    assert_eq!(applied, Applied::ServerRestart);
    assert!(store.meta().is_none());
    assert!(store.latest().is_none());

    // Subsequent messages resume normal application.
    assert_eq!(
        store.apply(ServerMessage::MatchStart {
            seq: 2,
            meta: test_meta(),
        }),
        Applied::MatchStart
    );
    assert!(store.meta().is_some());
}
