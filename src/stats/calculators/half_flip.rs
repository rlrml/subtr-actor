use super::*;

#[path = "half_flip_calculator.rs"]
mod half_flip_calculator;
#[path = "half_flip_candidate.rs"]
mod half_flip_candidate;
#[path = "half_flip_candidate_event.rs"]
mod half_flip_candidate_event;
#[path = "half_flip_candidate_start.rs"]
mod half_flip_candidate_start;
#[path = "half_flip_candidate_update.rs"]
mod half_flip_candidate_update;
#[path = "half_flip_event.rs"]
mod half_flip_event;
#[path = "half_flip_stats.rs"]
mod half_flip_stats;
#[path = "half_flip_update.rs"]
mod half_flip_update;
#[path = "half_flip_update_finalize.rs"]
mod half_flip_update_finalize;

const HALF_FLIP_EVALUATION_SECONDS: f32 = 0.65;
const HALF_FLIP_MAX_CANDIDATE_SECONDS: f32 = 1.0;
const HALF_FLIP_MAX_START_Z: f32 = PLAYER_GROUND_Z_THRESHOLD + 45.0;
const HALF_FLIP_MIN_START_SPEED: f32 = 250.0;
const HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT: f32 = 0.55;
const HALF_FLIP_MIN_REORIENTATION_ALIGNMENT: f32 = 0.60;
const HALF_FLIP_MIN_FORWARD_REVERSAL: f32 = 0.55;
const HALF_FLIP_MIN_FORWARD_VERTICAL: f32 = 0.22;
const HALF_FLIP_MIN_CONFIDENCE: f32 = 0.55;
const HALF_FLIP_HIGH_CONFIDENCE: f32 = 0.78;

pub use half_flip_calculator::HalfFlipCalculator;
pub(super) use half_flip_candidate::ActiveHalfFlipCandidate;
pub use half_flip_event::HalfFlipEvent;
pub use half_flip_stats::HalfFlipStats;

#[cfg(test)]
#[path = "half_flip_tests.rs"]
mod tests;
