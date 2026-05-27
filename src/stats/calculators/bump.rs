use super::*;

#[path = "bump_accessors.rs"]
mod bump_accessors;
#[path = "bump_detect.rs"]
mod bump_detect;
#[path = "bump_evaluate.rs"]
mod bump_evaluate;
#[path = "bump_evaluate_event.rs"]
mod bump_evaluate_event;
#[path = "bump_evaluate_selection.rs"]
mod bump_evaluate_selection;
#[path = "bump_event.rs"]
mod bump_event;
#[path = "bump_geometry.rs"]
mod bump_geometry;
#[path = "bump_record.rs"]
mod bump_record;
#[path = "bump_state.rs"]
mod bump_state;
#[path = "bump_stats.rs"]
mod bump_stats;
#[path = "bump_suppression.rs"]
mod bump_suppression;
#[path = "bump_suppression_fifty.rs"]
mod bump_suppression_fifty;
#[path = "bump_update.rs"]
mod bump_update;

pub use bump_event::BumpEvent;
pub use bump_state::BumpCalculator;
use bump_state::{DirectionalBumpCandidate, PreviousPlayerSample};
pub use bump_stats::{BumpPlayerStats, BumpTeamStats};

const BUMP_MAX_SAMPLE_DT: f32 = 0.18;
const BUMP_MAX_CONTACT_DISTANCE: f32 = 230.0;
const BUMP_MAX_VERTICAL_GAP: f32 = 190.0;
const BUMP_MIN_CLOSING_SPEED: f32 = 420.0;
const BUMP_MIN_VICTIM_IMPULSE: f32 = 180.0;
const BUMP_MIN_INITIATOR_SLOWDOWN: f32 = 100.0;
const BUMP_MIN_DIRECTIONAL_SCORE: f32 = 650.0;
const BUMP_MIN_SCORE_MARGIN: f32 = 175.0;
const BUMP_REPEAT_FRAME_WINDOW: usize = 10;
const BUMP_FIFTY_FIFTY_SUPPRESSION_WINDOW_SECONDS: f32 = 0.35;

#[cfg(test)]
#[path = "bump_tests.rs"]
mod tests;
