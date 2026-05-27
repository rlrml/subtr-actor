use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_time: f32,
    pub start_frame: usize,
    pub duration: f32,
    pub start_ball_position: [f32; 3],
    pub end_ball_position: [f32; 3],
    pub ball_travel_distance: f32,
    pub ball_advance_distance: f32,
    pub lateral_centering_distance: f32,
}
