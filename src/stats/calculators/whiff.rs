use super::*;

#[path = "whiff_accessors.rs"]
mod whiff_accessors;
#[path = "whiff_candidate.rs"]
mod whiff_candidate;
#[path = "whiff_event.rs"]
mod whiff_event;
#[path = "whiff_hitbox.rs"]
mod whiff_hitbox;
#[path = "whiff_record.rs"]
mod whiff_record;
#[path = "whiff_state.rs"]
mod whiff_state;
#[path = "whiff_stats.rs"]
mod whiff_stats;
#[path = "whiff_stats_sync.rs"]
mod whiff_stats_sync;
#[path = "whiff_touch.rs"]
mod whiff_touch;
#[path = "whiff_update.rs"]
mod whiff_update;
#[path = "whiff_update_candidates.rs"]
mod whiff_update_candidates;

pub use whiff_event::{WhiffEvent, WhiffEventKind};
use whiff_state::ActiveWhiffCandidate;
pub use whiff_state::WhiffCalculator;
pub use whiff_stats::WhiffStats;

const WHIFF_ENTER_DISTANCE: f32 = 150.0;
const WHIFF_EXIT_DISTANCE: f32 = 285.0;
const WHIFF_MAX_CANDIDATE_SECONDS: f32 = 0.65;
const WHIFF_MIN_APPROACH_SPEED: f32 = 700.0;
const WHIFF_MIN_CLOSING_SPEED: f32 = 450.0;
const WHIFF_MIN_FORWARD_ALIGNMENT: f32 = 0.55;
const WHIFF_MIN_VELOCITY_ALIGNMENT: f32 = 0.7;
const WHIFF_MIN_DODGE_APPROACH_SPEED: f32 = 450.0;
const WHIFF_MIN_DODGE_CLOSING_SPEED: f32 = 300.0;
const WHIFF_MIN_DODGE_FORWARD_ALIGNMENT: f32 = 0.25;
const WHIFF_MAX_LATERAL_OFFSET: f32 = 120.0;
const WHIFF_MAX_DODGE_LATERAL_OFFSET: f32 = 150.0;
const WHIFF_MIN_LOCAL_FORWARD_OFFSET: f32 = 0.0;
const WHIFF_MIN_DODGE_LOCAL_FORWARD_OFFSET: f32 = -20.0;

const WHIFF_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("dodge_state", "no_dodge"),
    StatLabel::new("dodge_state", "dodge"),
];

fn whiff_dodge_state_label(dodge_active: bool) -> StatLabel {
    if dodge_active {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

#[cfg(test)]
#[path = "whiff_tests.rs"]
mod tests;
