use super::*;

#[path = "one_timer_calculator.rs"]
mod one_timer_calculator;
#[path = "one_timer_detection.rs"]
mod one_timer_detection;
#[path = "one_timer_event.rs"]
mod one_timer_event;
#[path = "one_timer_record.rs"]
mod one_timer_record;
#[path = "one_timer_stats.rs"]
mod one_timer_stats;
#[path = "one_timer_update.rs"]
mod one_timer_update;

const ONE_TIMER_MIN_BALL_SPEED: f32 = 1000.0;
const ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.65;
const GOAL_CENTER_Y: f32 = 5120.0;

pub use one_timer_calculator::OneTimerCalculator;
pub use one_timer_event::OneTimerEvent;
pub use one_timer_stats::{OneTimerPlayerStats, OneTimerTeamStats};

#[cfg(test)]
#[path = "one_timer_tests.rs"]
mod tests;
