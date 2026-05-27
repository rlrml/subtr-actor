use super::*;

#[path = "dodge_reset_calculator.rs"]
mod dodge_reset_calculator;
#[path = "dodge_reset_confirm.rs"]
mod dodge_reset_confirm;
#[path = "dodge_reset_event.rs"]
mod dodge_reset_event;
#[path = "dodge_reset_on_ball.rs"]
mod dodge_reset_on_ball;
#[path = "dodge_reset_pending.rs"]
mod dodge_reset_pending;
#[path = "dodge_reset_stats.rs"]
mod dodge_reset_stats;
#[path = "dodge_reset_update.rs"]
mod dodge_reset_update;

const FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS: f32 = 0.05;
const FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS: f32 = 2.0;
const FLIP_RESET_GROUNDED_Z: f32 = 80.0;

pub use dodge_reset_calculator::DodgeResetCalculator;
pub use dodge_reset_event::{ConfirmedFlipResetEvent, DodgeResetEvent};
pub use dodge_reset_stats::DodgeResetStats;

#[cfg(test)]
#[path = "dodge_reset_tests.rs"]
mod tests;
