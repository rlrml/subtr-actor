#[path = "comparable_types_structs_boost.rs"]
mod boost;
#[path = "comparable_types_structs_core_demo.rs"]
mod core_demo;
#[path = "comparable_types_structs_movement.rs"]
mod movement;
#[path = "comparable_types_structs_positioning.rs"]
mod positioning;
#[path = "comparable_types_structs_replay.rs"]
mod replay;

pub(crate) use boost::ComparableBoostStats;
pub(crate) use core_demo::{ComparableCoreStats, ComparableDemoStats};
pub(crate) use movement::ComparableMovementStats;
pub(crate) use positioning::ComparablePositioningStats;
pub(crate) use replay::{ComparablePlayerStats, ComparableReplayStats, ComparableTeamStats};
