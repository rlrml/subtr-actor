use super::*;

#[path = "fifty_fifty_calculator.rs"]
mod fifty_fifty_calculator;
#[path = "fifty_fifty_detection.rs"]
mod fifty_fifty_detection;
#[path = "fifty_fifty_event.rs"]
mod fifty_fifty_event;
#[path = "fifty_fifty_labels.rs"]
mod fifty_fifty_labels;
#[path = "fifty_fifty_player_stats.rs"]
mod fifty_fifty_player_stats;
#[path = "fifty_fifty_player_stats_sync.rs"]
mod fifty_fifty_player_stats_sync;
#[path = "fifty_fifty_state_types.rs"]
mod fifty_fifty_state_types;
#[path = "fifty_fifty_stats.rs"]
mod fifty_fifty_stats;
#[path = "fifty_fifty_stats_sync.rs"]
mod fifty_fifty_stats_sync;
#[path = "fifty_fifty_team_stats.rs"]
mod fifty_fifty_team_stats;

pub use fifty_fifty_calculator::FiftyFiftyCalculator;
pub use fifty_fifty_event::FiftyFiftyEvent;
use fifty_fifty_labels::{
    fifty_fifty_phase_label, fifty_fifty_player_outcome_label, fifty_fifty_player_possession_label,
    fifty_fifty_possession_label, fifty_fifty_team_one_dodge_state_label,
    fifty_fifty_team_outcome_label, fifty_fifty_team_zero_dodge_state_label,
    fifty_fifty_touch_dodge_state_label,
};
pub use fifty_fifty_player_stats::FiftyFiftyPlayerStats;
pub use fifty_fifty_state_types::{ActiveFiftyFifty, FiftyFiftyState};
pub use fifty_fifty_stats::FiftyFiftyStats;
pub use fifty_fifty_team_stats::FiftyFiftyTeamStats;

pub(crate) const FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS: f32 = 0.2;
pub(crate) const FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS: f32 = 0.35;
pub(crate) const FIFTY_FIFTY_MAX_DURATION_SECONDS: f32 = 1.25;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_DISTANCE: f32 = 180.0;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_SPEED: f32 = 220.0;
