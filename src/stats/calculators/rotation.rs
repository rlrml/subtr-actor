use super::*;

const DEFAULT_ROLE_DEPTH_MARGIN: f32 = 150.0;
const DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN: f32 = 250.0;
const DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS: f32 = 0.35;

#[path = "rotation_calculator.rs"]
mod rotation_calculator;
#[path = "rotation_depth.rs"]
mod rotation_depth;
#[path = "rotation_emit.rs"]
mod rotation_emit;
#[path = "rotation_events.rs"]
mod rotation_events;
#[path = "rotation_inactive.rs"]
mod rotation_inactive;
#[path = "rotation_player_stats.rs"]
mod rotation_player_stats;
#[path = "rotation_player_time.rs"]
mod rotation_player_time;
#[path = "rotation_player_update.rs"]
mod rotation_player_update;
#[path = "rotation_scoring.rs"]
mod rotation_scoring;
#[path = "rotation_states.rs"]
mod rotation_states;
#[path = "rotation_stint_state.rs"]
mod rotation_stint_state;
#[path = "rotation_stints.rs"]
mod rotation_stints;
#[path = "rotation_team_access.rs"]
mod rotation_team_access;
#[path = "rotation_team_change.rs"]
mod rotation_team_change;
#[path = "rotation_team_invalid.rs"]
mod rotation_team_invalid;
#[path = "rotation_team_orphans.rs"]
mod rotation_team_orphans;
#[path = "rotation_team_stats.rs"]
mod rotation_team_stats;
#[path = "rotation_team_update.rs"]
mod rotation_team_update;
#[path = "rotation_tracker.rs"]
mod rotation_tracker;
#[path = "rotation_update.rs"]
mod rotation_update;

pub use rotation_calculator::{RotationCalculator, RotationCalculatorConfig};
pub use rotation_events::{RotationPlayerEvent, RotationTeamEvent};
pub use rotation_player_stats::RotationPlayerStats;
pub use rotation_states::{PlayDepthState, RoleState};
pub use rotation_team_stats::RotationTeamStats;

use rotation_events::RotationPlayerEventState;
use rotation_player_time::{add_depth_time, add_role_time};
use rotation_stint_state::FirstManStintState;
use rotation_tracker::TeamFirstManTracker;

#[cfg(test)]
#[path = "rotation_tests.rs"]
mod tests;
