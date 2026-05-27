use super::*;
#[path = "boost_accessors.rs"]
mod boost_accessors;
#[path = "boost_accounting.rs"]
mod boost_accounting;
#[path = "boost_accounting_collected.rs"]
mod boost_accounting_collected;
#[path = "boost_accounting_inactive.rs"]
mod boost_accounting_inactive;
#[path = "boost_accounting_respawn.rs"]
mod boost_accounting_respawn;
#[path = "boost_accounting_warnings.rs"]
mod boost_accounting_warnings;
#[path = "boost_events.rs"]
mod boost_events;
#[path = "boost_labels.rs"]
mod boost_labels;
#[path = "boost_pad_infer_details.rs"]
mod boost_pad_infer_details;
#[path = "boost_pad_infer_index.rs"]
mod boost_pad_infer_index;
#[path = "boost_pad_nearest.rs"]
mod boost_pad_nearest;
#[path = "boost_pad_resolve.rs"]
mod boost_pad_resolve;
#[path = "boost_pad_resolve_amounts.rs"]
mod boost_pad_resolve_amounts;
#[path = "boost_pad_resolve_ledger.rs"]
mod boost_pad_resolve_ledger;
#[path = "boost_pad_resolve_stats.rs"]
mod boost_pad_resolve_stats;
#[path = "boost_pad_resolve_stats_extra.rs"]
mod boost_pad_resolve_stats_extra;
#[path = "boost_pads.rs"]
mod boost_pads;
#[path = "boost_pickup_classify.rs"]
mod boost_pickup_classify;
#[path = "boost_pickup_comparison.rs"]
mod boost_pickup_comparison;
#[path = "boost_pickup_queue.rs"]
mod boost_pickup_queue;
#[path = "boost_pickups.rs"]
mod boost_pickups;
#[path = "boost_state.rs"]
mod boost_state;
#[path = "boost_stats.rs"]
mod boost_stats;
#[path = "boost_stats_methods.rs"]
mod boost_stats_methods;
#[path = "boost_types.rs"]
mod boost_types;
#[path = "boost_update.rs"]
mod boost_update;
#[path = "boost_update_context.rs"]
mod boost_update_context;
#[path = "boost_update_pad_events.rs"]
mod boost_update_pad_events;
#[path = "boost_update_pad_pickup_active.rs"]
mod boost_update_pad_pickup_active;
#[path = "boost_update_pad_pickup_active_helpers.rs"]
mod boost_update_pad_pickup_active_helpers;
#[path = "boost_update_pad_pickup_active_resolve.rs"]
mod boost_update_pad_pickup_active_resolve;
#[path = "boost_update_pad_pickup_inactive.rs"]
mod boost_update_pad_pickup_inactive;
#[path = "boost_update_player.rs"]
mod boost_update_player;
#[path = "boost_update_player_inferred.rs"]
mod boost_update_player_inferred;
#[path = "boost_update_player_level_times.rs"]
mod boost_update_player_level_times;
#[path = "boost_update_player_levels.rs"]
mod boost_update_player_levels;
#[path = "boost_update_player_respawns.rs"]
mod boost_update_player_respawns;
#[path = "boost_update_sample.rs"]
mod boost_update_sample;
#[path = "boost_update_used.rs"]
mod boost_update_used;
#[path = "boost_update_used_allocation.rs"]
mod boost_update_used_allocation;
#[path = "boost_update_used_totals.rs"]
mod boost_update_used_totals;
use boost_events::PendingBoostPickupEvent;
pub use boost_events::*;
use boost_labels::*;
use boost_pad_resolve_amounts::ResolvedPickupAmounts;
pub use boost_state::{BoostCalculator, BoostCalculatorConfig};
use boost_state::{
    BoostInvariantWarningKey, BoostLedgerContext, PendingBoostPickup, PendingDemoRespawn,
};
pub use boost_stats::BoostStats;
use boost_types::BoostIncreaseReason;
pub use boost_types::{BoostPickupPadType::*, *};
const DEMO_RESPAWN_WINDOW_SECONDS: f32 = 3.2;
#[cfg(test)]
#[path = "boost_tests.rs"]
mod tests;
