use super::*;

#[path = "match_stats_accessors.rs"]
mod match_stats_accessors;
#[path = "match_stats_boost_leadup.rs"]
mod match_stats_boost_leadup;
#[path = "match_stats_context.rs"]
mod match_stats_context;
#[path = "match_stats_core_events.rs"]
mod match_stats_core_events;
#[path = "match_stats_delta.rs"]
mod match_stats_delta;
#[path = "match_stats_delta_core.rs"]
mod match_stats_delta_core;
#[path = "match_stats_delta_goal.rs"]
mod match_stats_delta_goal;
#[path = "match_stats_delta_helpers.rs"]
mod match_stats_delta_helpers;
#[path = "match_stats_delta_player_fields.rs"]
mod match_stats_delta_player_fields;
#[path = "match_stats_delta_scoring.rs"]
mod match_stats_delta_scoring;
#[path = "match_stats_delta_sort.rs"]
mod match_stats_delta_sort;
#[path = "match_stats_events.rs"]
mod match_stats_events;
#[path = "match_stats_finish.rs"]
mod match_stats_finish;
#[path = "match_stats_goal_air_time.rs"]
mod match_stats_goal_air_time;
#[path = "match_stats_goal_buildup.rs"]
mod match_stats_goal_buildup;
#[path = "match_stats_goal_buildup_classify.rs"]
mod match_stats_goal_buildup_classify;
#[path = "match_stats_goal_context.rs"]
mod match_stats_goal_context;
#[path = "match_stats_goal_context_events.rs"]
mod match_stats_goal_context_events;
#[path = "match_stats_goal_context_types.rs"]
mod match_stats_goal_context_types;
#[path = "match_stats_goal_touch.rs"]
mod match_stats_goal_touch;
#[path = "match_stats_kickoff.rs"]
mod match_stats_kickoff;
#[path = "match_stats_player_context.rs"]
mod match_stats_player_context;
#[path = "match_stats_state.rs"]
mod match_stats_state;
#[path = "match_stats_team_stats.rs"]
mod match_stats_team_stats;
#[path = "match_stats_update.rs"]
mod match_stats_update;
#[path = "match_stats_update_events.rs"]
mod match_stats_update_events;
#[path = "match_stats_update_frame.rs"]
mod match_stats_update_frame;
#[path = "match_stats_update_goals.rs"]
mod match_stats_update_goals;
#[path = "match_stats_update_players.rs"]
mod match_stats_update_players;
#[path = "match_stats_update_team_scores.rs"]
mod match_stats_update_team_scores;

pub use match_stats_context::*;
pub use match_stats_events::*;
pub use match_stats_goal_context_types::*;
use match_stats_state::*;

const GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS: f32 = 10.0;
const GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS: f32 = 20.0;
const GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS: f32 = 40.0;
const GOAL_BUILDUP_LOOKBACK_SECONDS: f32 = 12.0;
const COUNTER_ATTACK_MAX_ATTACK_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS: f32 = 1.0;
const SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS: f32 = 6.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS: f32 = 7.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS: f32 = 3.5;
const GOAL_CONTEXT_BOOST_LEADUP_SECONDS: f32 = 5.0;
const BALL_GROUND_CONTACT_MAX_Z: f32 = BALL_RADIUS_Z + 5.0;

#[derive(Debug, Clone, Default)]
pub struct MatchStatsCalculator {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    last_emitted_player_stats: HashMap<PlayerId, CorePlayerStats>,
    last_emitted_team_zero_stats: CoreTeamStats,
    last_emitted_team_one_stats: CoreTeamStats,
    core_player_events: Vec<CorePlayerStatsEvent>,
    core_team_events: Vec<CoreTeamStatsEvent>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<PendingGoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
    kickoff_waiting_for_first_touch: bool,
    active_kickoff_touch_time: Option<f32>,
    goal_buildup_samples: Vec<GoalBuildupSample>,
    goal_buildup_pressure_events: Vec<GoalBuildupPressureEvent>,
    goal_context_events: Vec<GoalContextEvent>,
    last_touch_context_by_player: HashMap<PlayerId, GoalTouchContext>,
    boost_leadup_samples_by_player: HashMap<PlayerId, VecDeque<BoostLeadupSample>>,
    last_ball_ground_contact_time: Option<f32>,
}

#[cfg(test)]
#[path = "match_stats_tests.rs"]
mod tests;
