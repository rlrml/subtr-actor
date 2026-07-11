//! Versioned wire protocol for the live game-state export stream.
//!
//! # Version rule
//!
//! - `protocol_major` must match the server's [`PROTOCOL_MAJOR`] exactly, for
//!   every client.
//! - For [`Encoding::Postcard`] clients, `protocol_minor` must **also** match
//!   [`PROTOCOL_MINOR`] exactly: postcard is not self-describing, so even an
//!   additive field change desynchronizes the byte stream.
//! - [`Encoding::Json`] tolerates minor-version drift; unknown fields are
//!   ignorable and JSON consumers can feature-detect.
//!
//! # Handshake
//!
//! [`ClientMessage::Hello`] is **always** sent as a JSON text frame (so
//! browsers and zero-dependency consumers can speak it). The server replies in
//! the negotiated encoding thereafter: `Postcard` messages travel as binary
//! WebSocket frames, `Json` messages as text frames.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use subtr_actor::LivePlayState;

use crate::meta::LiveMatchMeta;
use crate::model::LiveFrame;
use crate::wire::{WireEventHistory, WireFrameEventsState};

/// Wire protocol major version; must match exactly for every client.
pub const PROTOCOL_MAJOR: u16 = 1;
/// Wire protocol minor version; must match exactly for postcard clients.
pub const PROTOCOL_MINOR: u16 = 0;

/// Recommended TCP port for live game-state export servers.
///
/// This is a *convention*, not a bind default: the server itself keeps the
/// standard `port == 0` ephemeral semantics. Hosts (e.g. the BakkesMod
/// state-export plugin cvar) should default to this value so consumers can
/// find the stream without configuration.
pub const DEFAULT_STATE_EXPORT_PORT: u16 = 49109;

/// The encoding a client negotiates for server messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encoding {
    /// Compact binary; sent as WebSocket binary frames. Not self-describing.
    Postcard,
    /// JSON; sent as WebSocket text frames.
    Json,
}

impl Encoding {
    /// Whether this encoding requires an exact `protocol_minor` match (see the
    /// module-level version rule).
    pub fn requires_exact_minor(self) -> bool {
        matches!(self, Encoding::Postcard)
    }
}

/// Checks a client's advertised protocol version against this build, applying
/// the module-level version rule for the requested encoding.
pub fn protocol_versions_compatible(
    encoding: Encoding,
    protocol_major: u16,
    protocol_minor: u16,
) -> bool {
    protocol_major == PROTOCOL_MAJOR
        && (!encoding.requires_exact_minor() || protocol_minor == PROTOCOL_MINOR)
}

/// Messages sent by clients. `Hello` is always JSON text (see module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello {
        protocol_major: u16,
        protocol_minor: u16,
        /// Encoding the server should use for its messages after the Hello.
        encoding: Encoding,
        /// Optional per-client frame-rate cap; the server downsamples `Frame`
        /// messages so this client never exceeds the rate. Non-frame messages
        /// are never dropped.
        max_frame_hz: Option<f32>,
    },
}

/// One live frame plus everything derived from it.
///
/// The server runs the event generator itself and ships the *derived* per-frame
/// events, so consumers never need generator state and late joiners can resume
/// from a snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FramePayload {
    pub frame: LiveFrame,
    pub derived_events: WireFrameEventsState,
    pub live_play: LivePlayState,
}

/// Messages broadcast by the server.
///
/// `seq` is a server-lifetime monotonically increasing message counter. Each
/// client's stream is strictly increasing in `seq`; gaps appear only when a
/// message was intentionally not sent to that client (frame downsampling, or
/// seqs consumed by another client's subscription messages).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// First message after a successful subscription.
    ServerInfo {
        protocol_major: u16,
        protocol_minor: u16,
        server: String,
        seq: u64,
    },
    /// A match roster was first observed.
    MatchStart {
        seq: u64,
        meta: LiveMatchMeta,
    },
    /// The roster changed mid-match (join/leave/rename/team change).
    RosterChange {
        seq: u64,
        meta: LiveMatchMeta,
    },
    /// Cumulative event history at subscription time, so mid-match joiners
    /// start from the same accumulated state as clients connected from the
    /// beginning.
    EventHistorySnapshot {
        seq: u64,
        history: WireEventHistory,
        latest_frame: Option<Box<FramePayload>>,
    },
    Frame {
        seq: u64,
        payload: Box<FramePayload>,
    },
    MatchEnd {
        seq: u64,
    },
    /// Emitted periodically when no frames are flowing. `time` is unix epoch
    /// seconds on the server.
    Heartbeat {
        seq: u64,
        time: f64,
    },
}

impl ServerMessage {
    pub fn seq(&self) -> u64 {
        match self {
            ServerMessage::ServerInfo { seq, .. }
            | ServerMessage::MatchStart { seq, .. }
            | ServerMessage::RosterChange { seq, .. }
            | ServerMessage::EventHistorySnapshot { seq, .. }
            | ServerMessage::Frame { seq, .. }
            | ServerMessage::MatchEnd { seq }
            | ServerMessage::Heartbeat { seq, .. } => *seq,
        }
    }

    /// Encodes this message; JSON is returned as UTF-8 text bytes.
    pub fn encode(&self, encoding: Encoding) -> Result<Vec<u8>, ProtocolError> {
        encode_with(self, encoding)
    }

    pub fn decode(encoding: Encoding, bytes: &[u8]) -> Result<Self, ProtocolError> {
        decode_with(encoding, bytes)
    }
}

impl ClientMessage {
    /// Encodes this message; JSON is returned as UTF-8 text bytes.
    pub fn encode(&self, encoding: Encoding) -> Result<Vec<u8>, ProtocolError> {
        encode_with(self, encoding)
    }

    pub fn decode(encoding: Encoding, bytes: &[u8]) -> Result<Self, ProtocolError> {
        decode_with(encoding, bytes)
    }

    /// Decodes a Hello-style JSON text payload (the handshake is always JSON).
    pub fn decode_json(text: &str) -> Result<Self, ProtocolError> {
        Ok(serde_json::from_str(text)?)
    }
}

fn encode_with<T: Serialize>(value: &T, encoding: Encoding) -> Result<Vec<u8>, ProtocolError> {
    match encoding {
        Encoding::Postcard => Ok(postcard::to_stdvec(value)?),
        Encoding::Json => Ok(serde_json::to_vec(value)?),
    }
}

fn decode_with<T: DeserializeOwned>(encoding: Encoding, bytes: &[u8]) -> Result<T, ProtocolError> {
    match encoding {
        Encoding::Postcard => Ok(postcard::from_bytes(bytes)?),
        Encoding::Json => Ok(serde_json::from_slice(bytes)?),
    }
}

/// Encoding/decoding failure for a protocol message.
#[derive(Debug)]
pub enum ProtocolError {
    Postcard(postcard::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::Postcard(error) => write!(formatter, "postcard: {error}"),
            ProtocolError::Json(error) => write!(formatter, "json: {error}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<postcard::Error> for ProtocolError {
    fn from(error: postcard::Error) -> Self {
        Self::Postcard(error)
    }
}

impl From<serde_json::Error> for ProtocolError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
