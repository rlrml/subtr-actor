use super::*;

#[derive(Debug, Clone)]
pub(super) struct PendingGoalEvent {
    pub(super) event: GoalEvent,
    pub(super) time_after_kickoff: Option<f32>,
    pub(super) goal_buildup: GoalBuildupKind,
    pub(super) ball_air_time_before_goal: Option<f32>,
}

#[derive(Debug, Clone)]
pub(super) struct GoalBuildupSample {
    pub(super) time: f32,
    pub(super) dt: f32,
    pub(super) ball_y: f32,
}

#[derive(Debug, Clone)]
pub(super) struct GoalBuildupPressureEvent {
    pub(super) time: f32,
    pub(super) is_team_0: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BoostLeadupSample {
    pub(super) time: f32,
    pub(super) boost_amount: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BoostLeadupStats {
    pub(super) average_boost: f32,
    pub(super) min_boost: f32,
}
