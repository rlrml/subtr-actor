use super::*;

#[path = "double_tap_backboard.rs"]
mod double_tap_backboard;
#[path = "double_tap_calculator.rs"]
mod double_tap_calculator;
#[path = "double_tap_event.rs"]
mod double_tap_event;
#[path = "double_tap_pending.rs"]
mod double_tap_pending;
#[path = "double_tap_stats.rs"]
mod double_tap_stats;
#[path = "double_tap_touch.rs"]
mod double_tap_touch;
#[path = "double_tap_update.rs"]
mod double_tap_update;

const DOUBLE_TAP_TOUCH_WINDOW_SECONDS: f32 = 2.5;

pub use double_tap_calculator::DoubleTapCalculator;
pub use double_tap_event::DoubleTapEvent;
pub(crate) use double_tap_pending::PendingBackboardBounce;
pub use double_tap_stats::{DoubleTapPlayerStats, DoubleTapTeamStats};
