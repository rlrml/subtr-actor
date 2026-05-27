use super::wall_aerial::{
    wall_aerial_normalize_score, wall_aerial_wall_for_position, WALL_AERIAL_HIGH_CONFIDENCE,
    WALL_AERIAL_MIN_TOUCH_BALL_Z, WALL_AERIAL_MIN_TOUCH_PLAYER_Z,
};
use super::*;

#[path = "wall_aerial_shot_calculator.rs"]
mod wall_aerial_shot_calculator;
#[path = "wall_aerial_shot_confidence.rs"]
mod wall_aerial_shot_confidence;
#[path = "wall_aerial_shot_event.rs"]
mod wall_aerial_shot_event;
#[path = "wall_aerial_shot_record.rs"]
mod wall_aerial_shot_record;
#[path = "wall_aerial_shot_shot.rs"]
mod wall_aerial_shot_shot;
#[path = "wall_aerial_shot_state.rs"]
mod wall_aerial_shot_state;
#[path = "wall_aerial_shot_stats.rs"]
mod wall_aerial_shot_stats;
#[path = "wall_aerial_shot_takeoff.rs"]
mod wall_aerial_shot_takeoff;
#[path = "wall_aerial_shot_update.rs"]
mod wall_aerial_shot_update;

const WALL_AERIAL_SHOT_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS: f32 = 2.25;
const WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS: f32 = 2.25;
const WALL_AERIAL_SHOT_GROUND_CONTACT_MAX_PLAYER_Z: f32 = 80.0;

pub use wall_aerial_shot_calculator::WallAerialShotCalculator;
pub(crate) use wall_aerial_shot_confidence::wall_aerial_shot_confidence;
pub use wall_aerial_shot_event::WallAerialShotEvent;
pub(crate) use wall_aerial_shot_state::{ArmedWallAerialShot, RecentWallContact};
pub use wall_aerial_shot_stats::WallAerialShotStats;

#[cfg(test)]
#[path = "wall_aerial_shot_tests.rs"]
mod tests;
