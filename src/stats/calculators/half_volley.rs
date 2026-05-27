use super::*;

#[path = "half_volley_bounce.rs"]
mod half_volley_bounce;
#[path = "half_volley_calculator.rs"]
mod half_volley_calculator;
#[path = "half_volley_config.rs"]
mod half_volley_config;
#[path = "half_volley_event.rs"]
mod half_volley_event;
#[path = "half_volley_movement.rs"]
mod half_volley_movement;
#[path = "half_volley_record.rs"]
mod half_volley_record;
#[path = "half_volley_stats.rs"]
mod half_volley_stats;
#[path = "half_volley_touch.rs"]
mod half_volley_touch;
#[path = "half_volley_update.rs"]
mod half_volley_update;

const DEFAULT_HALF_VOLLEY_MAX_BOUNCE_TO_TOUCH_SECONDS: f32 = 0.45;
const DEFAULT_HALF_VOLLEY_MIN_BALL_SPEED: f32 = 1000.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MAX_BALL_Z: f32 = BALL_RADIUS_Z + 45.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MIN_APPROACH_SPEED_Z: f32 = 250.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MIN_REBOUND_SPEED_Z: f32 = 150.0;
const HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.35;
const HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS: f32 = 0.45;
const HALF_VOLLEY_GOAL_CENTER_Y: f32 = 5120.0;

pub(crate) use half_volley_bounce::FloorBounce;
pub use half_volley_calculator::HalfVolleyCalculator;
pub use half_volley_config::HalfVolleyCalculatorConfig;
pub use half_volley_event::HalfVolleyEvent;
pub(crate) use half_volley_movement::{DodgeStart, GroundContact};
pub use half_volley_stats::{HalfVolleyPlayerStats, HalfVolleyTeamStats};

#[cfg(test)]
#[path = "half_volley_tests.rs"]
mod tests;
