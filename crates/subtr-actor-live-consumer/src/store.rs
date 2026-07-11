//! Client-side mirror of the server's broadcast state.
//!
//! [`LiveStateStore::apply`] folds each [`ServerMessage`] into owned state:
//! the current roster ([`LiveMatchMeta`] plus its derived
//! [`ReplayMeta`]), the cumulative [`LiveEventHistory`], and the latest
//! frame. [`LiveStateStore::view`] then exposes that state as the shared
//! [`LiveProcessorView`] so the same analysis pipeline that runs on the
//! server side (or on replays) can run over the streamed data.
//!
//! History maintenance mirrors the server exactly: an
//! `EventHistorySnapshot` replaces the history wholesale, and each `Frame`'s
//! derived events are appended through the same
//! [`LiveEventHistory::append_frame_events`] the producer path uses, so a
//! mid-stream joiner converges on the same accumulated state as a client
//! connected from the first frame.

use subtr_actor::{FrameEventsState, LivePlayState, PlayerId, ReplayMeta};
use subtr_actor_live::{
    LiveEventHistory, LiveFrame, LiveMatchMeta, LiveProcessorView, ServerMessage,
};

/// What applying one [`ServerMessage`] did to the store, at the granularity
/// the graph driver needs to react.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// `ServerInfo`; no state change beyond seq tracking.
    Info,
    MatchStart,
    RosterChange,
    /// `EventHistorySnapshot`; history replaced wholesale (and the latest
    /// frame adopted, when present).
    Snapshot,
    Frame,
    /// `MatchEnd`; match state cleared, mirroring the server reset.
    MatchEnd,
    Heartbeat,
    /// The message's `seq` went backwards: the server restarted. The store
    /// reset itself and then applied the message normally.
    ServerRestart,
}

/// The most recent frame and everything derived from it, as shipped in a
/// `Frame` message (or a snapshot's `latest_frame`).
#[derive(Debug, Clone)]
pub struct LatestFrame {
    pub frame: LiveFrame,
    pub derived_events: FrameEventsState,
    pub live_play: LivePlayState,
}

/// Owned live-match state reconstructed from the message stream.
#[derive(Default)]
pub struct LiveStateStore {
    last_seq: Option<u64>,
    meta: Option<LiveMatchMeta>,
    replay_meta: Option<ReplayMeta>,
    history: LiveEventHistory,
    latest: Option<LatestFrame>,
}

impl LiveStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Folds one server message into the store.
    ///
    /// A `seq` decrease relative to the previous message means the server
    /// restarted (its seq counter is server-lifetime monotonic); the store
    /// resets before applying and reports [`Applied::ServerRestart`] so the
    /// driver can reset the analysis graph too.
    pub fn apply(&mut self, message: ServerMessage) -> Applied {
        let seq = message.seq();
        let restarted = self.last_seq.is_some_and(|last| seq < last);
        if restarted {
            self.reset();
        }
        self.last_seq = Some(seq);

        let applied = match message {
            ServerMessage::ServerInfo { .. } => Applied::Info,
            ServerMessage::MatchStart { meta, .. } => {
                self.set_meta(meta);
                Applied::MatchStart
            }
            ServerMessage::RosterChange { meta, .. } => {
                self.set_meta(meta);
                Applied::RosterChange
            }
            ServerMessage::EventHistorySnapshot {
                history,
                latest_frame,
                ..
            } => {
                // The snapshot history already includes the snapshot's
                // latest frame's events (the server appends before
                // publishing), so the frame is adopted without re-appending.
                self.history = history.into();
                if let Some(payload) = latest_frame {
                    self.latest = Some(latest_frame_from_payload(*payload));
                }
                Applied::Snapshot
            }
            ServerMessage::Frame { payload, .. } => {
                let latest = latest_frame_from_payload(*payload);
                self.history.append_frame_events(&latest.derived_events);
                self.latest = Some(latest);
                Applied::Frame
            }
            ServerMessage::MatchEnd { .. } => {
                // Mirror the server's own MatchEnd reset (roster, history,
                // and latest frame all belong to the ended match).
                let last_seq = self.last_seq;
                self.reset();
                self.last_seq = last_seq;
                Applied::MatchEnd
            }
            ServerMessage::Heartbeat { .. } => Applied::Heartbeat,
        };

        if restarted {
            Applied::ServerRestart
        } else {
            applied
        }
    }

    fn set_meta(&mut self, meta: LiveMatchMeta) {
        self.replay_meta = Some(meta.replay_meta());
        self.meta = Some(meta);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    /// Roster of the current match, when one has been observed.
    pub fn meta(&self) -> Option<&LiveMatchMeta> {
        self.meta.as_ref()
    }

    /// [`ReplayMeta`] derived from the current roster.
    pub fn replay_meta(&self) -> Option<&ReplayMeta> {
        self.replay_meta.as_ref()
    }

    /// Cumulative event history for the current match.
    pub fn history(&self) -> &LiveEventHistory {
        &self.history
    }

    /// The most recently applied frame.
    pub fn latest(&self) -> Option<&LatestFrame> {
        self.latest.as_ref()
    }

    /// Resolves a player id to its roster name, for display.
    pub fn player_name(&self, player_id: &PlayerId) -> Option<&str> {
        self.meta
            .as_ref()?
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(|player| player.name.as_deref())
    }

    /// Builds the shared [`LiveProcessorView`] over the stored state, or
    /// `None` before the first frame.
    ///
    /// The view owns its [`LiveFrame`] (and per-frame events), so both are
    /// cloned per call; frames are small (a couple of KB), which is cheap
    /// even at full frame rate.
    pub fn view(&self) -> Option<LiveProcessorView<'_>> {
        let latest = self.latest.as_ref()?;
        Some(LiveProcessorView::new(
            self.replay_meta.as_ref(),
            latest.frame.clone(),
            latest.derived_events.clone(),
            &self.history,
        ))
    }
}

fn latest_frame_from_payload(payload: subtr_actor_live::FramePayload) -> LatestFrame {
    LatestFrame {
        frame: payload.frame,
        derived_events: payload.derived_events.into(),
        live_play: payload.live_play,
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
