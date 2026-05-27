use super::*;

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y: f32 = -1200.0;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y: f32 = -250.0;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y: f32 = 2200.0;
const DEFAULT_LEVEL_BALL_DEPTH_MARGIN: f32 = 150.0;

#[path = "positioning_ball_depth.rs"]
mod positioning_ball_depth;
#[path = "positioning_calculator.rs"]
mod positioning_calculator;
#[path = "positioning_event.rs"]
mod positioning_event;
#[path = "positioning_event_delta.rs"]
mod positioning_event_delta;
#[path = "positioning_goal_events.rs"]
mod positioning_goal_events;
#[path = "positioning_live_player_sample.rs"]
mod positioning_live_player_sample;
#[path = "positioning_player_sample.rs"]
mod positioning_player_sample;
#[path = "positioning_player_totals.rs"]
mod positioning_player_totals;
#[path = "positioning_sample.rs"]
mod positioning_sample;
#[path = "positioning_stats_methods.rs"]
mod positioning_stats_methods;
#[path = "positioning_stats_types.rs"]
mod positioning_stats_types;
#[path = "positioning_team_ball_rank.rs"]
mod positioning_team_ball_rank;
#[path = "positioning_team_distance.rs"]
mod positioning_team_distance;
#[path = "positioning_team_role_time.rs"]
mod positioning_team_role_time;
#[path = "positioning_team_roles.rs"]
mod positioning_team_roles;
#[path = "positioning_team_sample.rs"]
mod positioning_team_sample;

#[cfg(test)]
pub(crate) use positioning_ball_depth::ball_depth_fractions;
pub use positioning_calculator::{PositioningCalculator, PositioningCalculatorConfig};
pub use positioning_event::PositioningEvent;
pub use positioning_stats_types::PositioningStats;

#[cfg(test)]
#[path = "positioning_tests.rs"]
mod tests;
