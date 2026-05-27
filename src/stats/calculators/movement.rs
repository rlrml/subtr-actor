use super::*;

#[path = "movement_apply.rs"]
mod movement_apply;
#[path = "movement_calculator.rs"]
mod movement_calculator;
#[path = "movement_classification.rs"]
mod movement_classification;
#[path = "movement_event.rs"]
mod movement_event;
#[path = "movement_stats.rs"]
mod movement_stats;
#[path = "movement_stats_complete.rs"]
mod movement_stats_complete;
#[path = "movement_update.rs"]
mod movement_update;

pub(crate) use movement_apply::{apply_movement_stats, movement_event};
pub use movement_calculator::MovementCalculator;
pub(crate) use movement_classification::{
    MovementClassification, MovementSpeedBand, ALL_MOVEMENT_SPEED_BANDS,
};
pub use movement_event::MovementEvent;
pub use movement_stats::MovementStats;
