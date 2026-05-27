use super::*;

#[path = "flick_accessors.rs"]
mod flick_accessors;
#[path = "flick_confidence.rs"]
mod flick_confidence;
#[path = "flick_control_observation.rs"]
mod flick_control_observation;
#[path = "flick_dodge.rs"]
mod flick_dodge;
#[path = "flick_event.rs"]
mod flick_event;
#[path = "flick_event_build.rs"]
mod flick_event_build;
#[path = "flick_record.rs"]
mod flick_record;
#[path = "flick_setup.rs"]
mod flick_setup;
#[path = "flick_setup_update.rs"]
mod flick_setup_update;
#[path = "flick_state.rs"]
mod flick_state;
#[path = "flick_stats.rs"]
mod flick_stats;
#[path = "flick_update.rs"]
mod flick_update;

pub use flick_event::FlickEvent;
pub use flick_state::FlickCalculator;
use flick_state::{ActiveFlickSetup, FlickControlObservation, FlickSetupSummary, RecentDodgeStart};
pub use flick_stats::FlickStats;

const FLICK_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.32;
const FLICK_MAX_CONTROL_TO_DODGE_SECONDS: f32 = 0.08;
const FLICK_MAX_SETUP_STALE_SECONDS: f32 = 0.35;
const FLICK_MIN_SETUP_SECONDS: f32 = 0.30;
const FLICK_MIN_BALL_SPEED_CHANGE: f32 = 450.0;
const FLICK_HIGH_CONFIDENCE: f32 = 0.80;
const FLICK_MIN_CONFIDENCE: f32 = 0.55;
const FLICK_MAX_CONTROL_BALL_Z: f32 = 700.0;
const FLICK_MAX_CONTROL_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.7;
const FLICK_MIN_CONTROL_VERTICAL_GAP: f32 = 35.0;
const FLICK_MAX_CONTROL_VERTICAL_GAP: f32 = 280.0;
const FLICK_MIN_LOCAL_Z: f32 = 20.0;
const FLICK_MAX_LOCAL_X_BEHIND: f32 = 95.0;
const FLICK_MAX_LOCAL_X_FRONT: f32 = 210.0;
const FLICK_MAX_LOCAL_Y: f32 = 170.0;
const FLICK_MIN_IMPULSE_AWAY_ALIGNMENT: f32 = 0.15;

#[cfg(test)]
#[path = "flick_tests.rs"]
mod tests;
