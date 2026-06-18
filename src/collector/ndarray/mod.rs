//! Dense numeric feature extraction: build an [`ndarray::Array2`] of replay
//! features for ML/analytics.
//!
//! # What feature adders are
//!
//! A [`NDArrayCollector`] runs the replay frame by frame and, for each sampled
//! frame, asks a list of *feature adders* to append their numbers to that
//! frame's row. Each adder owns a fixed set of named columns and knows how to
//! compute them from the current [`ProcessorView`](crate::ProcessorView) (and,
//! for analysis-backed adders, from analysis-graph state). The collector
//! concatenates every adder's columns into one wide matrix plus a header list,
//! so the set of adders you choose *is* your feature schema.
//!
//! There are two flavours, each in a global and a per-player form:
//!
//! - [`FeatureAdder`] / [`PlayerFeatureAdder`] — compute directly from frame and
//!   processor state.
//! - [`AnalysisFeatureAdder`] / [`AnalysisPlayerFeatureAdder`] — additionally
//!   read [`AnalysisGraph`](crate::stats::analysis_graph::AnalysisGraph) state,
//!   declaring their node dependencies so the collector wires up the graph.
//!
//! The `LengthChecked*` trait variants let an adder fix its column count at
//! compile time. Player adders emit their columns once per player.
//!
//! # Registering adders
//!
//! - **Explicitly:** construct adders and pass them to the collector (see the
//!   builder methods on [`NDArrayCollector`]).
//! - **By name:** [`NDArrayCollector::from_strings`] /
//!   [`from_strings_typed`](NDArrayCollector::from_strings_typed) look up adders
//!   in a built-in string registry — convenient for the Python/JS bindings.
//!
//! Recognized global (ball / game) names include `BallRigidBody`,
//! `BallRigidBodyNoVelocities`, `BallRigidBodyQuaternions`,
//! `BallRigidBodyBasis`, `InterpolatedBallRigidBodyNoVelocities`,
//! `CurrentTime`, `FrameTime`, `SecondsRemaining`, and `BallHasBeenHit`.
//! Recognized per-player names include `PlayerRigidBody`,
//! `PlayerRigidBodyNoVelocities`, `PlayerRelativeBallPosition`,
//! `PlayerLocalRelativeBallVelocity`, `PlayerBoost`, `PlayerJump`,
//! `PlayerAnyJump`, `PlayerDodgeRefreshed`, `PlayerDemolishedBy`, and
//! `PlayerBallDistance`. Analysis-backed per-player event features are also
//! addressable by mechanic name (e.g. `touch`, `flick`, `whiff`, `bump`,
//! `pass`, `rotation`, `movement`, `positioning`). The matching arms in
//! `collector.rs` and `analysis_builtins.rs` are the authoritative list.

mod analysis_builtins;
mod builtins;
mod collector;
mod traits;

pub use self::analysis_builtins::*;
pub use self::builtins::*;
pub use self::collector::*;
pub use self::traits::*;
