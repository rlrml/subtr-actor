use super::*;

#[path = "ceiling_shot_calculator.rs"]
mod ceiling_shot_calculator;
#[path = "ceiling_shot_candidate_confidence.rs"]
mod ceiling_shot_candidate_confidence;
#[path = "ceiling_shot_candidate_event.rs"]
mod ceiling_shot_candidate_event;
#[path = "ceiling_shot_candidate_metrics.rs"]
mod ceiling_shot_candidate_metrics;
#[path = "ceiling_shot_contact.rs"]
mod ceiling_shot_contact;
#[path = "ceiling_shot_event.rs"]
mod ceiling_shot_event;
#[path = "ceiling_shot_record.rs"]
mod ceiling_shot_record;
#[path = "ceiling_shot_stats.rs"]
mod ceiling_shot_stats;
#[path = "ceiling_shot_update.rs"]
mod ceiling_shot_update;

const SOCCAR_CEILING_Z: f32 = 2044.0;
const CEILING_CONTACT_MAX_GAP: f32 = 90.0;
const CEILING_CONTACT_MIN_ROOF_ALIGNMENT: f32 = 0.72;
const CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS: f32 = 1.35;
const CEILING_SHOT_MIN_TOUCH_SEPARATION: f32 = 120.0;
const CEILING_SHOT_MIN_PLAYER_HEIGHT: f32 = 260.0;
const CEILING_SHOT_MIN_BALL_HEIGHT: f32 = 220.0;
const CEILING_SHOT_MIN_FORWARD_ALIGNMENT: f32 = 0.12;
const CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED: f32 = 90.0;
const CEILING_SHOT_MIN_BALL_SPEED_CHANGE: f32 = 120.0;
const CEILING_SHOT_MIN_CONFIDENCE: f32 = 0.54;
const CEILING_SHOT_HIGH_CONFIDENCE: f32 = 0.78;

pub use ceiling_shot_calculator::CeilingShotCalculator;
pub(super) use ceiling_shot_candidate_metrics::CeilingShotCandidateMetrics;
pub(super) use ceiling_shot_contact::RecentCeilingContact;
pub use ceiling_shot_event::CeilingShotEvent;
pub use ceiling_shot_stats::CeilingShotStats;
