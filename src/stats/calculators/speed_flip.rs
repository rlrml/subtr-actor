use super::*;

#[path = "speed_flip_accessors.rs"]
mod speed_flip_accessors;
#[path = "speed_flip_candidate_event.rs"]
mod speed_flip_candidate_event;
#[path = "speed_flip_candidate_event_helpers.rs"]
mod speed_flip_candidate_event_helpers;
#[path = "speed_flip_candidate_start.rs"]
mod speed_flip_candidate_start;
#[path = "speed_flip_candidate_update.rs"]
mod speed_flip_candidate_update;
#[path = "speed_flip_candidate_update_helpers.rs"]
mod speed_flip_candidate_update_helpers;
#[path = "speed_flip_event.rs"]
mod speed_flip_event;
#[path = "speed_flip_finalize.rs"]
mod speed_flip_finalize;
#[path = "speed_flip_kickoff.rs"]
mod speed_flip_kickoff;
#[path = "speed_flip_record.rs"]
mod speed_flip_record;
#[path = "speed_flip_scoring.rs"]
mod speed_flip_scoring;
#[path = "speed_flip_state.rs"]
mod speed_flip_state;
#[path = "speed_flip_stats.rs"]
mod speed_flip_stats;
#[path = "speed_flip_update.rs"]
mod speed_flip_update;

pub use speed_flip_event::SpeedFlipEvent;
use speed_flip_state::ActiveSpeedFlipCandidate;
pub use speed_flip_state::SpeedFlipCalculator;
pub use speed_flip_stats::SpeedFlipStats;

const SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS: f32 = 1.1;
const SPEED_FLIP_EVALUATION_SECONDS: f32 = 0.32;
const SPEED_FLIP_MAX_CANDIDATE_SECONDS: f32 = 0.55;
const SPEED_FLIP_MAX_GROUND_Z: f32 = 80.0;
const SPEED_FLIP_KICKOFF_MOTION_SPEED: f32 = 100.0;
const SPEED_FLIP_MIN_ALIGNMENT: f32 = 0.72;
const SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS: f32 = 0.18;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA: f32 = 80.0;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT: f32 = 0.35;
const SPEED_FLIP_MIN_CONFIDENCE: f32 = 0.45;
const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;

#[cfg(test)]
#[path = "speed_flip_tests.rs"]
mod tests;
