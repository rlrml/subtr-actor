use super::*;

#[path = "territorial_pressure_accessors.rs"]
mod territorial_pressure_accessors;
#[path = "territorial_pressure_active.rs"]
mod territorial_pressure_active;
#[path = "territorial_pressure_candidate.rs"]
mod territorial_pressure_candidate;
#[path = "territorial_pressure_config.rs"]
mod territorial_pressure_config;
#[path = "territorial_pressure_event.rs"]
mod territorial_pressure_event;
#[path = "territorial_pressure_finish.rs"]
mod territorial_pressure_finish;
#[path = "territorial_pressure_start.rs"]
mod territorial_pressure_start;
#[path = "territorial_pressure_state.rs"]
mod territorial_pressure_state;
#[path = "territorial_pressure_stats.rs"]
mod territorial_pressure_stats;
#[path = "territorial_pressure_stats_update.rs"]
mod territorial_pressure_stats_update;
#[path = "territorial_pressure_stats_update_fields.rs"]
mod territorial_pressure_stats_update_fields;
#[path = "territorial_pressure_team_counts.rs"]
mod territorial_pressure_team_counts;
#[path = "territorial_pressure_team_stats.rs"]
mod territorial_pressure_team_stats;
#[path = "territorial_pressure_update.rs"]
mod territorial_pressure_update;

pub use territorial_pressure_config::TerritorialPressureCalculatorConfig;
pub use territorial_pressure_event::{TerritorialPressureEndReason, TerritorialPressureEvent};
pub use territorial_pressure_state::TerritorialPressureCalculator;
use territorial_pressure_state::{
    ActiveTerritorialPressureSession, CandidateTerritorialPressureSession,
};
pub use territorial_pressure_stats::TerritorialPressureStats;
pub use territorial_pressure_team_stats::TerritorialPressureTeamStats;

const DEFAULT_TERRITORIAL_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;
const DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_SECONDS: f32 = 2.0;
const DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS: f32 = 0.75;
const DEFAULT_TERRITORIAL_PRESSURE_RELIEF_GRACE_SECONDS: f32 = 3.0;
const DEFAULT_TERRITORIAL_PRESSURE_CONFIRMED_RELIEF_GRACE_SECONDS: f32 = 1.25;

#[cfg(test)]
#[path = "territorial_pressure_tests.rs"]
mod tests;
