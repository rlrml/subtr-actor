//! Consumer side of the subtr-actor live game-state export stream.
//!
//! Three layers, composable but independent:
//!
//! - [`LiveClient`]: blocking WebSocket client that performs the
//!   `Hello`/`ServerInfo` handshake and yields decoded
//!   [`ServerMessage`](subtr_actor_live::ServerMessage)s.
//! - [`LiveStateStore`]: folds messages into owned match state (roster,
//!   cumulative event history, latest frame) and exposes it as the shared
//!   [`LiveProcessorView`](subtr_actor_live::LiveProcessorView).
//! - [`LiveGraphDriver`]: runs the full stats analysis graph over the stored
//!   state and drains normalized timeline events, mirroring the BakkesMod
//!   live FFI's frame loop.
//!
//! Frames arrive with their derived per-frame events attached
//! (`FramePayload::derived_events`); the server already ran the live event
//! generator, so consumers must never re-run it.

#![allow(clippy::result_large_err)]

mod client;
mod driver;
mod store;

pub use client::*;
pub use driver::*;
pub use store::*;
