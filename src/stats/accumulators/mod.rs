//! Accumulators: plain structs that fold a calculator's per-frame events into
//! running totals over a replay.
//!
//! Unlike calculators and analysis nodes, accumulators share no trait — each is
//! a data container that holds counts, durations, distances, and confidence
//! sums for one mechanic or play type. They typically come in a per-player /
//! per-team / match-wide trio plus a `*StatsAccumulator` that applies events.
//! Downstream, each accumulator's published values are defined by its
//! [`StatFieldProvider`] impl in
//! [`crate::stats::export`].
//!
//! Browse the full set in the item list below; representative examples are
//! [`BoostStats`], [`TouchStats`], [`MovementStats`], [`PositioningStats`],
//! [`KickoffStats`], [`FiftyFiftyStats`], and [`CorePlayerStats`] /
//! [`CoreTeamStats`] (core scoreboard).

pub(crate) use std::collections::HashMap;

pub(crate) use serde::{Deserialize, Serialize};

pub(crate) use crate::stats::calculators::*;
pub(crate) use crate::stats::common::*;
pub(crate) use crate::*;

#[cfg(test)]
pub(crate) use test_projection::*;

pub mod backboard;
pub use backboard::*;
pub mod air_dribble;
pub use air_dribble::*;
pub mod ball_carry;
pub use ball_carry::*;
pub mod boost;
pub use boost::*;
pub mod boost_invariants;
pub use boost_invariants::*;
pub mod bump;
pub use bump::*;
pub mod ceiling_shot;
pub use ceiling_shot::*;
pub mod center;
pub use center::*;
pub mod controlled_play;
pub use controlled_play::*;
pub mod demo;
pub use demo::*;
pub mod dodge_reset;
pub use dodge_reset::*;
pub mod double_tap;
pub use double_tap::*;
pub mod fifty_fifty;
pub use fifty_fifty::*;
pub mod flick;
pub use flick::*;
pub mod half_flip;
pub use half_flip::*;
pub mod half_volley;
pub use half_volley::*;
pub mod kickoff;
pub use kickoff::*;
pub mod match_stats;
pub use match_stats::*;
pub mod movement;
pub use movement::*;
pub mod musty_flick;
pub use musty_flick::*;
pub mod one_timer;
pub use one_timer::*;
pub mod pass;
pub use pass::*;
pub mod positioning;
pub use positioning::*;
pub mod possession;
pub use possession::*;
pub mod powerslide;
pub use powerslide::*;
pub mod ball_half;
pub use ball_half::*;
pub mod ball_third;
pub use ball_third::*;
pub mod rotation;
pub use rotation::*;
pub mod rush;
pub use rush::*;
pub mod speed_flip;
pub use speed_flip::*;
pub mod territorial_pressure;
pub use territorial_pressure::*;
pub mod touch;
pub use touch::*;
pub mod wall_aerial_shot;
pub use wall_aerial_shot::*;
pub mod wall_aerial;
pub use wall_aerial::*;
pub mod wavedash;
pub use wavedash::*;
pub mod whiff;
pub use whiff::*;

#[cfg(test)]
mod test_projection;
