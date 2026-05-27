use crate::*;
use serde::Serialize;
use std::collections::HashMap;

#[path = "flip_reset_builders.rs"]
mod flip_reset_builders;
#[path = "flip_reset_candidates.rs"]
mod flip_reset_candidates;
#[path = "flip_reset_collector.rs"]
mod flip_reset_collector;
#[path = "flip_reset_confidence.rs"]
mod flip_reset_confidence;
#[path = "flip_reset_dodge_edges.rs"]
mod flip_reset_dodge_edges;
#[path = "flip_reset_events.rs"]
mod flip_reset_events;
#[path = "flip_reset_followup_builders.rs"]
mod flip_reset_followup_builders;
#[path = "flip_reset_followup_emit.rs"]
mod flip_reset_followup_emit;
#[path = "flip_reset_followup_prune.rs"]
mod flip_reset_followup_prune;
#[path = "flip_reset_followup_update.rs"]
mod flip_reset_followup_update;
#[path = "flip_reset_heuristic.rs"]
mod flip_reset_heuristic;
#[path = "flip_reset_proximity.rs"]
mod flip_reset_proximity;
#[path = "flip_reset_proximity_builders.rs"]
mod flip_reset_proximity_builders;
#[path = "flip_reset_proximity_update.rs"]
mod flip_reset_proximity_update;
#[path = "flip_reset_tracker.rs"]
mod flip_reset_tracker;
#[path = "flip_reset_tracker_accessors.rs"]
mod flip_reset_tracker_accessors;
#[path = "flip_reset_update_events.rs"]
mod flip_reset_update_events;
#[path = "flip_reset_wall_dodge.rs"]
mod flip_reset_wall_dodge;
#[path = "flip_reset_wall_state.rs"]
mod flip_reset_wall_state;
#[path = "flip_reset_wall_tracking.rs"]
mod flip_reset_wall_tracking;

pub(crate) use flip_reset_candidates::{flip_reset_candidate, flip_reset_followup_touch_candidate};
pub use flip_reset_events::{
    DodgeRefreshedEvent, FlipResetEvent, FlipResetFollowupDodgeEvent, PostWallDodgeEvent,
};
pub use flip_reset_tracker::FlipResetTracker;

use flip_reset_confidence::flip_reset_confidence;
use flip_reset_heuristic::{
    build_touch_features, scale_factor_for_positions, FlipResetHeuristic, FlipResetTouchFeatures,
};
use flip_reset_proximity::flip_reset_proximity_candidate;
