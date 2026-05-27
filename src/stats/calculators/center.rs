use super::*;

#[path = "center_calculator.rs"]
mod center_calculator;
#[path = "center_detection.rs"]
mod center_detection;
#[path = "center_disqualification.rs"]
mod center_disqualification;
#[path = "center_event.rs"]
mod center_event;
#[path = "center_pending_touch.rs"]
mod center_pending_touch;
#[path = "center_record.rs"]
mod center_record;
#[path = "center_stats.rs"]
mod center_stats;
#[path = "center_update.rs"]
mod center_update;

const CENTER_MAX_DURATION_SECONDS: f32 = 3.0;
const CENTER_MIN_BALL_TRAVEL_DISTANCE: f32 = 500.0;
const CENTER_MIN_LATERAL_DISTANCE: f32 = 500.0;
const CENTER_MIN_START_ABS_X: f32 = 1600.0;
const CENTER_MAX_END_ABS_X: f32 = 1400.0;
const CENTER_MIN_START_ATTACKING_Y: f32 = BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const CENTER_MIN_END_ATTACKING_Y: f32 = FIELD_ZONE_BOUNDARY_Y;

pub use center_calculator::CenterCalculator;
pub use center_event::CenterEvent;
pub(crate) use center_pending_touch::PendingCenterTouch;
pub use center_stats::{CenterPlayerStats, CenterTeamStats};

#[cfg(test)]
#[path = "center_tests.rs"]
mod tests;
