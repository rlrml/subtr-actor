use super::*;

#[path = "musty_flick_calculator.rs"]
mod musty_flick_calculator;
#[path = "musty_flick_candidate_confidence.rs"]
mod musty_flick_candidate_confidence;
#[path = "musty_flick_candidate_event.rs"]
mod musty_flick_candidate_event;
#[path = "musty_flick_candidate_metrics.rs"]
mod musty_flick_candidate_metrics;
#[path = "musty_flick_candidate_spin.rs"]
mod musty_flick_candidate_spin;
#[path = "musty_flick_dodge_start.rs"]
mod musty_flick_dodge_start;
#[path = "musty_flick_event.rs"]
mod musty_flick_event;
#[path = "musty_flick_record.rs"]
mod musty_flick_record;
#[path = "musty_flick_stats.rs"]
mod musty_flick_stats;
#[path = "musty_flick_update.rs"]
mod musty_flick_update;

const MUSTY_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.22;
const MUSTY_MIN_PLAYER_HEIGHT: f32 = 80.0;
const MUSTY_AERIAL_HEIGHT: f32 = 180.0;
const MUSTY_MIN_FORWARD_APPROACH_SPEED: f32 = 150.0;
const MUSTY_MIN_BALL_SPEED_CHANGE: f32 = 150.0;
const MUSTY_MIN_REAR_ALIGNMENT: f32 = 0.15;
const MUSTY_MIN_TOP_ALIGNMENT: f32 = 0.10;
const MUSTY_MIN_LOCAL_Z: f32 = 5.0;
const MUSTY_MAX_LOCAL_X: f32 = 60.0;
const MUSTY_MAX_LOCAL_Y: f32 = 170.0;
const MUSTY_MIN_PITCH_RATE: f32 = 2.5;
const MUSTY_MIN_PITCH_DOMINANCE_RATIO: f32 = 1.1;
const MUSTY_MIN_DODGE_START_FORWARD_Z: f32 = -0.25;
const MUSTY_MIN_CONFIDENCE: f32 = 0.55;
const MUSTY_HIGH_CONFIDENCE: f32 = 0.80;

pub use musty_flick_calculator::MustyFlickCalculator;
pub(crate) use musty_flick_candidate_metrics::MustyFlickCandidateMetrics;
pub(crate) use musty_flick_candidate_spin::MustyFlickCandidateSpin;
pub(crate) use musty_flick_dodge_start::RecentDodgeStart;
pub use musty_flick_event::MustyFlickEvent;
pub use musty_flick_stats::MustyFlickStats;
