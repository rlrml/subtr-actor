#![allow(clippy::result_large_err)]
#![doc = include_str!("lib_docs.md")]

pub mod collector;
pub mod constants;
pub mod error;
pub mod geometry;
pub mod mechanics;
pub mod playlist_generation;
pub mod processor;
pub mod replay_meta;
pub mod replay_plausibility;
pub mod replay_types;
pub mod search;
pub mod stats;
pub mod ts_bindings;
pub mod vec_map;

pub mod actor_state {
    //! Compatibility re-export for processor actor-state types.
    pub use crate::processor::actor_state::*;
}

pub use crate::actor_state::*;
pub use crate::collector::*;
pub use crate::constants::*;
pub use crate::error::*;
pub use crate::geometry::*;
pub use crate::mechanics::*;
pub use crate::playlist_generation::*;
pub use crate::processor::*;
pub use crate::replay_meta::*;
pub use crate::replay_plausibility::*;
pub use crate::replay_types::*;
pub use crate::search::*;
pub use crate::stats::*;
pub(crate) use crate::vec_map::*;
