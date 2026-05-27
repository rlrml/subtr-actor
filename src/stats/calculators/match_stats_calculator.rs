use super::*;

#[derive(Debug, Clone, Default)]
pub struct MatchStatsCalculator {
    pub(super) player_stats: HashMap<PlayerId, CorePlayerStats>,
    pub(super) player_teams: HashMap<PlayerId, bool>,
    pub(super) previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    pub(super) last_emitted_player_stats: HashMap<PlayerId, CorePlayerStats>,
    pub(super) last_emitted_team_zero_stats: CoreTeamStats,
    pub(super) last_emitted_team_one_stats: CoreTeamStats,
    pub(super) core_player_events: Vec<CorePlayerStatsEvent>,
    pub(super) core_team_events: Vec<CoreTeamStatsEvent>,
    pub(super) timeline: Vec<TimelineEvent>,
    pub(super) pending_goal_events: Vec<PendingGoalEvent>,
    pub(super) previous_team_scores: Option<(i32, i32)>,
    pub(super) kickoff_waiting_for_first_touch: bool,
    pub(super) active_kickoff_touch_time: Option<f32>,
    pub(super) goal_buildup_samples: Vec<GoalBuildupSample>,
    pub(super) goal_buildup_pressure_events: Vec<GoalBuildupPressureEvent>,
    pub(super) goal_context_events: Vec<GoalContextEvent>,
    pub(super) last_touch_context_by_player: HashMap<PlayerId, GoalTouchContext>,
    pub(super) boost_leadup_samples_by_player: HashMap<PlayerId, VecDeque<BoostLeadupSample>>,
    pub(super) last_ball_ground_contact_time: Option<f32>,
}
