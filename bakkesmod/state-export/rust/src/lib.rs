//! C ABI shell that exports live Rocket League game state from a BakkesMod
//! plugin over the `subtr-actor-live` WebSocket protocol.
//!
//! This crate is deliberately thin: repr(C) frames sampled by the C++ plugin
//! ([`abi`]) are converted to the owned [`subtr_actor_live::LiveFrame`] model
//! ([`convert`]) and pushed into a
//! [`subtr_actor_live::server::ServerHandle`] ([`ffi`]). No stats graph and
//! no analysis run in-process — consumers (e.g. `subtr-actor-live-consumer`)
//! drive the analysis graph on their side of the socket.
//!
//! The ABI mirrors the conventions of the sibling `subtr-actor-bakkesmod`
//! and `replay-to-training` crates: an opaque handle (`SeEngine`),
//! `len`/`write` pairs for strings, and a C header
//! (`include/state_export.h`) whose struct layouts are locked by the tests
//! in `src/lib_tests/`.

#![allow(clippy::missing_safety_doc)]

use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

use boxcars::{Quaternion, RemoteId, RigidBody, SwitchId, Vector3f};
use subtr_actor_live::{
    DEFAULT_STATE_EXPORT_PORT, LiveBoostPadEvent, LiveBoostPadEventKind, LiveCameraState,
    LiveControllerInput, LiveDemolishEvent, LiveDodgeRefreshedEvent, LiveEventTiming,
    LiveExplicitEvents, LiveExportServer, LiveExportServerConfig, LiveFrame, LiveGoalEvent,
    LiveMatchContext, LiveMatchStats, LivePlayerFrame, LivePlayerStatEvent,
    LivePlayerStatEventKind, LiveTouchEvent, ServerHandle, player_id,
};

mod abi;
mod convert;
mod ffi;

pub use abi::*;
pub(crate) use convert::*;
pub use ffi::*;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
