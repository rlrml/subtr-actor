use super::*;

#[path = "air_dribble_origin.rs"]
mod air_dribble_origin;
#[path = "air_dribble_policy.rs"]
mod air_dribble_policy;
#[path = "air_dribble_stats.rs"]
mod air_dribble_stats;

const AIR_DRIBBLE_MIN_BALL_Z: f32 = 300.0;
pub(crate) const AIR_DRIBBLE_MIN_PLAYER_Z: f32 = 100.0;
const AIR_DRIBBLE_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 3.0;
const AIR_DRIBBLE_MAX_ABOVE_CAR_GAP: f32 = 360.0;
const AIR_DRIBBLE_MAX_BELOW_CAR_GAP: f32 = 100.0;
pub(crate) const AIR_DRIBBLE_MIN_DURATION: f32 = 0.65;
const AIR_DRIBBLE_MIN_TOUCHES: u32 = 3;
const AIR_DRIBBLE_MIN_AIR_TOUCHES: u32 = 2;
const WALL_TAKEOFF_MIN_Z: f32 = 120.0;
const SIDE_WALL_START_ABS_X: f32 = 3200.0;
const BACK_WALL_START_ABS_Y: f32 = 4600.0;

pub use air_dribble_origin::AirDribbleOrigin;
pub(crate) use air_dribble_origin::{air_dribble_origin_label, AIR_DRIBBLE_ORIGIN_LABELS};
pub(crate) use air_dribble_policy::AirDribblePolicy;
pub use air_dribble_stats::AirDribbleStats;

#[cfg(test)]
#[path = "air_dribble_tests.rs"]
mod tests;
