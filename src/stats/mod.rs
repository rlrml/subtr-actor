#![allow(ambiguous_glob_reexports)]
//! Replay analysis: detect gameplay mechanics, accumulate stats, and export
//! them.
//!
//! This module ties together four layers:
//!
//! - [`analysis_graph`] — the [`AnalysisNode`](analysis_graph::AnalysisNode)
//!   dependency DAG that drives everything. Start here for the runtime model.
//! - **Calculators** — the detection logic wrapped by analysis nodes. Each
//!   detects one mechanic or maintains one piece of derived state and emits
//!   typed events implementing [`StatsEvent`]. They are re-exported at this
//!   module's root (the `*Calculator` / `*Event` types in the item list below).
//! - [`accumulators`] — plain structs that fold a calculator's events into
//!   running per-player / per-team / per-match totals over the replay.
//! - [`export`] — the report-facing stat-field model: each accumulator
//!   implements [`StatFieldProvider`] to publish its
//!   values as labeled, unit-tagged [`ExportedStat`]s.
//! - [`timeline`] — assembles per-frame stat timelines for playback UIs.
//!
//! # The calculators
//!
//! Calculators group by what they produce (browse them via the [`StatsEvent`]
//! *Implementors* list, or the `*Calculator` entries in the item list):
//!
//! - **Mechanics** — [`FlickCalculator`], [`HalfFlipCalculator`],
//!   [`SpeedFlipCalculator`], [`WavedashCalculator`],
//!   [`PowerslideCalculator`], [`FlipImpulseCalculator`],
//!   [`DodgeResetCalculator`], [`WallAerialCalculator`],
//!   [`WallAerialShotCalculator`], [`CeilingShotCalculator`],
//!   [`DoubleTapCalculator`], [`HalfVolleyCalculator`], [`OneTimerCalculator`],
//!   [`BallCarryCalculator`], [`AirDribbleCalculator`].
//! - **Play & contests** — [`TouchCalculator`], [`PassCalculator`],
//!   [`CenterCalculator`], [`KickoffCalculator`], [`BumpCalculator`],
//!   [`DemoCalculator`], [`RushCalculator`], [`ControlledPlayCalculator`],
//!   [`TerritorialPressureCalculator`], [`WhiffCalculator`],
//!   [`FiftyFiftyCalculator`], [`BackboardCalculator`].
//! - **Derived state** — [`PossessionCalculator`],
//!   [`PlayerPossessionCalculator`], [`PositioningCalculator`],
//!   [`RotationCalculator`], [`MovementCalculator`], [`BoostCalculator`],
//!   [`PlayerVerticalStateCalculator`], [`LivePlayTracker`].
//! - **Match-level** — [`MatchStatsCalculator`] and the goal-tag calculators
//!   (the `*GoalCalculator` types).
//!
//! See the [stats-runtime guide](crate::guides::calculators_and_analysis_nodes)
//! and the [confidence guide](crate::guides::stat_confidence).

pub mod accumulators;
pub mod analysis_graph;
pub(crate) mod calculators;
pub(crate) mod common;
pub mod labels;
pub mod timeline;

pub use accumulators::*;
pub use calculators::*;
pub use labels::*;
pub use timeline::*;
