//! C ABI for saving replay-captured shots into Rocket League custom
//! training pack (`.tem`) files.
//!
//! This crate backs the `bakkesmod-replay-to-training/` BakkesMod plugin: the C++
//! side captures ball and car rigid-body state from the in-game replay
//! viewer and hands it across the C ABI defined in [`ffi`]; this crate turns
//! each captured shot into a training-pack round (via the typed-archetype
//! mapping in [`archetypes`]) and serializes the pack with
//! [`subtr_actor_training`].
//!
//! The ABI mirrors the conventions of the existing `subtr-actor-bakkesmod`
//! crate: an opaque handle (`TrPack`), `len`/`write` pairs for strings, and
//! a C header (`include/replay_to_training.h`) whose struct layouts are locked by
//! the tests in `lib_tests.rs`.

pub mod abi;
pub mod archetypes;
pub mod ffi;
pub mod recorder;

pub use abi::{TrBallState, TrCapturedShot, TrCarState, TrRotator, TrVec3};
pub use ffi::TrPack;
pub use recorder::RecorderPack;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
