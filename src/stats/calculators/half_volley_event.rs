use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub bounce_time: f32,
    pub bounce_frame: usize,
    pub bounce_to_touch_seconds: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}
