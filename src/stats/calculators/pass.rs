use super::*;

#[path = "pass_calculator.rs"]
mod pass_calculator;
#[path = "pass_detection.rs"]
mod pass_detection;
#[path = "pass_event.rs"]
mod pass_event;
#[path = "pass_fifty_fifty.rs"]
mod pass_fifty_fifty;
#[path = "pass_kind.rs"]
mod pass_kind;
#[path = "pass_pending_touch.rs"]
mod pass_pending_touch;
#[path = "pass_record.rs"]
mod pass_record;
#[path = "pass_stats.rs"]
mod pass_stats;
#[path = "pass_update.rs"]
mod pass_update;

const PASS_MAX_DURATION_SECONDS: f32 = 3.5;
const PASS_MIN_BALL_TRAVEL_DISTANCE: f32 = 500.0;

pub use pass_calculator::PassCalculator;
pub use pass_event::{PassEvent, PassLastCompletedEvent};
pub use pass_kind::PassKind;
pub(crate) use pass_pending_touch::PendingPassTouch;
pub use pass_stats::{PassPlayerStats, PassTeamStats};

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
