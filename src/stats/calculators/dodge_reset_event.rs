use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub on_ball: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfirmedFlipResetEvent {
    pub time: f32,
    pub frame: usize,
    pub reset_time: f32,
    pub reset_frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub time_since_reset: f32,
}
