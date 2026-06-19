//! Per-frame state updaters for [`ReplayProcessor`](crate::ReplayProcessor).
//!
//! Each submodule here adds an `impl ReplayProcessor` block with one or more
//! `update_*` methods that advance a slice of processor state for the current
//! frame. There is no shared trait — the processor's frame loop simply calls
//! each updater in a fixed order:
//!
//! - `boost` — boost amounts and boost-pad pickup/respawn events.
//! - `goals` — goal detection and scoreboard updates.
//! - `demolishes` — demolition (kill/death) events.
//! - `camera` — coalesced per-player camera/driving state changes.
//! - `player_stats` — replay-reported player stat events.
//! - `tracking` — actor tracking and rigid-body bookkeeping.
//! - `mappings` — actor/object id mappings used by the queries.
//!
//! These modules are crate-internal; the resulting state is read by collectors
//! through [`ProcessorView`](crate::ProcessorView).

use super::*;

mod boost;
mod camera;
mod demolishes;
mod goals;
mod mappings;
mod player_stats;
mod tracking;
