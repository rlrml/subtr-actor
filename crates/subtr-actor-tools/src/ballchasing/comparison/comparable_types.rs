#[path = "comparable_types_compare_boost.rs"]
mod compare_boost;
#[path = "comparable_types_compare_core.rs"]
mod compare_core;
#[path = "comparable_types_compare_demo.rs"]
mod compare_demo;
#[path = "comparable_types_compare_hierarchy.rs"]
mod compare_hierarchy;
#[path = "comparable_types_compare_movement.rs"]
mod compare_movement;
#[path = "comparable_types_compare_positioning.rs"]
mod compare_positioning;
#[path = "comparable_types_structs.rs"]
mod structs;

pub(crate) use structs::ComparableReplayStats;
pub(super) use structs::{
    ComparableBoostStats, ComparableCoreStats, ComparableDemoStats, ComparableMovementStats,
    ComparablePlayerStats, ComparablePositioningStats,
};
