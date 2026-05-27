use super::*;

#[path = "wall_aerial_accessors.rs"]
mod wall_aerial_accessors;
#[path = "wall_aerial_control.rs"]
mod wall_aerial_control;
#[path = "wall_aerial_event.rs"]
mod wall_aerial_event;
#[path = "wall_aerial_event_build.rs"]
mod wall_aerial_event_build;
#[path = "wall_aerial_event_parts.rs"]
mod wall_aerial_event_parts;
#[path = "wall_aerial_record.rs"]
mod wall_aerial_record;
#[path = "wall_aerial_state.rs"]
mod wall_aerial_state;
#[path = "wall_aerial_takeoff.rs"]
mod wall_aerial_takeoff;
#[path = "wall_aerial_takeoff_setup.rs"]
mod wall_aerial_takeoff_setup;
#[path = "wall_aerial_takeoff_state.rs"]
mod wall_aerial_takeoff_state;
#[path = "wall_aerial_update.rs"]
mod wall_aerial_update;
#[path = "wall_aerial_wall.rs"]
mod wall_aerial_wall;

pub use wall_aerial_event::{WallAerialEvent, WallAerialStats};
pub use wall_aerial_state::WallAerialCalculator;
use wall_aerial_state::{
    ActiveWallControl, ArmedWallAerial, CompletedWallSetup, RecentWallContact, WallControl,
};
pub use wall_aerial_wall::WallAerialWall;
pub(crate) use wall_aerial_wall::{
    wall_aerial_goal_alignment, wall_aerial_normalize_score, wall_aerial_wall_for_position,
};

const WALL_AERIAL_MIN_CONTROL_DURATION: f32 = 0.30;
const WALL_AERIAL_MAX_CONTROL_BALL_DISTANCE: f32 = 380.0;
const WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS: f32 = 1.25;
const WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS: f32 = 2.25;
const WALL_AERIAL_MIN_SECONDS_BETWEEN_ATTEMPTS: f32 = 3.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;
const WALL_AERIAL_SETUP_SIDE_WALL_START_ABS_X: f32 = 3200.0;
const WALL_AERIAL_SETUP_BACK_WALL_START_ABS_Y: f32 = 4600.0;
const WALL_AERIAL_MIN_CONTINUATION_PLAYER_Z: f32 = 300.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_BALL_Z: f32 = 400.0;
const WALL_AERIAL_REFERENCE_BALL_SPEED_CHANGE: f32 = 80.0;
pub(crate) const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

#[cfg(test)]
#[path = "wall_aerial_tests.rs"]
mod tests;
