use crate::*;
use boxcars;
use std::collections::HashMap;

#[macro_use]
mod processor_macros;

pub mod actor_state;
pub mod view;
pub use actor_state::*;
pub use view::*;

mod bootstrap;
mod debug;
mod processor_actor_helpers;
mod processor_attribute_types;
mod processor_cached_ids;
mod processor_cached_lookup;
mod processor_frame;
mod processor_new;
mod processor_normalization;
mod processor_player_mappings;
mod processor_process;
mod processor_process_all;
mod processor_reset;
mod processor_rigid_body;
mod processor_struct;
mod queries;
mod updaters;

pub(crate) use processor_actor_helpers::{get_actor_id_from_active_actor, use_update_actor};
pub(crate) use processor_attribute_types::attribute_type_name;
pub(crate) use processor_cached_ids::CachedObjectIds;
pub use processor_struct::ReplayProcessor;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
