use super::SaVec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaGoalBuildupKind {
    CounterAttack = 1,
    SustainedPressure = 2,
    Other = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaGoalContextEvent {
    pub frame_number: u64,
    pub time: f32,
    pub scoring_team_is_team_0: u8,
    pub has_scorer: u8,
    pub scorer_index: u32,
    pub has_scoring_team_most_back_player: u8,
    pub scoring_team_most_back_player_index: u32,
    pub has_defending_team_most_back_player: u8,
    pub defending_team_most_back_player_index: u32,
    pub has_ball_position: u8,
    pub ball_position: SaVec3,
    pub has_ball_air_time_before_goal: u8,
    pub ball_air_time_before_goal: f32,
    pub goal_buildup: SaGoalBuildupKind,
}
