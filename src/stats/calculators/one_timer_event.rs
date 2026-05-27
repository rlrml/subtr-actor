use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    pub is_team_0: bool,
    pub pass_start_time: f32,
    pub pass_start_frame: usize,
    pub pass_duration: f32,
    pub pass_travel_distance: f32,
    pub pass_advance_distance: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}
