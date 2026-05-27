use super::*;

#[path = "wavedash_calculator.rs"]
mod wavedash_calculator;
#[path = "wavedash_candidate.rs"]
mod wavedash_candidate;
#[path = "wavedash_candidate_event.rs"]
mod wavedash_candidate_event;
#[path = "wavedash_event.rs"]
mod wavedash_event;
#[path = "wavedash_record.rs"]
mod wavedash_record;
#[path = "wavedash_stats.rs"]
mod wavedash_stats;
#[path = "wavedash_update.rs"]
mod wavedash_update;

const WAVEDASH_MAX_DODGE_TO_LANDING_SECONDS: f32 = 0.35;
const WAVEDASH_MAX_CANDIDATE_SECONDS: f32 = 0.5;
const WAVEDASH_MIN_DODGE_START_Z: f32 = PLAYER_GROUND_Z_THRESHOLD + 8.0;
const WAVEDASH_MAX_DODGE_START_Z: f32 = 320.0;
const WAVEDASH_MIN_LANDING_UPRIGHTNESS: f32 = 0.15;
const WAVEDASH_MIN_CONFIDENCE: f32 = 0.45;
const WAVEDASH_HIGH_CONFIDENCE: f32 = 0.75;

pub use wavedash_calculator::WavedashCalculator;
pub(crate) use wavedash_candidate::ActiveWavedashCandidate;
pub use wavedash_event::WavedashEvent;
pub use wavedash_stats::WavedashStats;

#[cfg(test)]
#[path = "wavedash_tests.rs"]
mod tests;
