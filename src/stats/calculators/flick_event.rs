use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub setup_start_time: f32,
    pub setup_start_frame: usize,
    pub setup_duration: f32,
    pub setup_touch_count: u32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub ball_speed_change: f32,
    pub ball_impulse: [f32; 3],
    pub impulse_away_alignment: f32,
    pub vertical_impulse: f32,
    pub confidence: f32,
}
