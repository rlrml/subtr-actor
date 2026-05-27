use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WavedashEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub dodge_position: [f32; 3],
    pub landing_position: [f32; 3],
    pub start_speed: f32,
    pub landing_speed: f32,
    pub horizontal_speed_gain: f32,
    pub landing_uprightness: f32,
    pub confidence: f32,
}
