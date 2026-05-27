use super::*;

#[path = "rush_accessors.rs"]
mod rush_accessors;
#[path = "rush_active.rs"]
mod rush_active;
#[path = "rush_config.rs"]
mod rush_config;
#[path = "rush_event.rs"]
mod rush_event;
#[path = "rush_labels.rs"]
mod rush_labels;
#[path = "rush_numbers.rs"]
mod rush_numbers;
#[path = "rush_record.rs"]
mod rush_record;
#[path = "rush_state.rs"]
mod rush_state;
#[path = "rush_stats.rs"]
mod rush_stats;
#[path = "rush_stats_sync.rs"]
mod rush_stats_sync;
#[path = "rush_team_stats.rs"]
mod rush_team_stats;
#[path = "rush_update.rs"]
mod rush_update;
#[path = "rush_update_parts.rs"]
mod rush_update_parts;

pub use rush_config::RushCalculatorConfig;
pub use rush_event::RushEvent;
use rush_labels::{rush_attackers_label, rush_defenders_label, rush_team_label};
use rush_state::ActiveRush;
pub use rush_state::RushCalculator;
pub use rush_stats::RushStats;
pub use rush_team_stats::RushTeamStats;

// Require the turnover to occur at least slightly inside the new attacking
// team's defensive half rather than anywhere around midfield.
const DEFAULT_RUSH_MAX_START_Y: f32 = -BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y: f32 = 900.0;
const DEFAULT_RUSH_DEFENDER_DISTANCE_Y: f32 = 150.0;
const DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS: f32 = 0.75;

#[cfg(test)]
#[path = "rush_tests.rs"]
mod tests;
