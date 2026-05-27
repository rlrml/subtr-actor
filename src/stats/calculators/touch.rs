use super::*;

const SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 320.0;
const HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 900.0;
const AERIAL_TOUCH_MIN_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;

#[path = "touch_apply_classification.rs"]
mod touch_apply_classification;
#[path = "touch_apply_events.rs"]
mod touch_apply_events;
#[path = "touch_ball_speed.rs"]
mod touch_ball_speed;
#[path = "touch_calculator.rs"]
mod touch_calculator;
#[path = "touch_classification.rs"]
mod touch_classification;
#[path = "touch_credit_movement.rs"]
mod touch_credit_movement;
#[path = "touch_event_build.rs"]
mod touch_event_build;
#[path = "touch_events.rs"]
mod touch_events;
#[path = "touch_fifty_fifty.rs"]
mod touch_fifty_fifty;
#[path = "touch_fifty_fifty_credit.rs"]
mod touch_fifty_fifty_credit;
#[path = "touch_labeled_counts.rs"]
mod touch_labeled_counts;
#[path = "touch_labels.rs"]
mod touch_labels;
#[path = "touch_last_touch.rs"]
mod touch_last_touch;
#[path = "touch_movement_credit.rs"]
mod touch_movement_credit;
#[path = "touch_sample.rs"]
mod touch_sample;
#[path = "touch_stats.rs"]
mod touch_stats;
#[path = "touch_stats_update.rs"]
mod touch_stats_update;
#[path = "touch_surface.rs"]
mod touch_surface;
#[path = "touch_update.rs"]
mod touch_update;

pub use touch_calculator::TouchCalculator;
pub use touch_events::{TouchBallMovementEvent, TouchLastTouchEvent, TouchStatsEvent};
pub use touch_stats::TouchStats;

use touch_calculator::PendingFiftyFiftyMovement;
use touch_classification::TouchClassification;
use touch_event_build::touch_stats_event;
use touch_labels::{
    TouchDodgeState, TouchKind, TouchSurface, ALL_TOUCH_DODGE_STATES, ALL_TOUCH_KINDS,
    ALL_TOUCH_SURFACES,
};

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
