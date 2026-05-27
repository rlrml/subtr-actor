use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MustyFlickEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub aerial: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub confidence: f32,
    pub local_ball_position: [f32; 3],
    pub rear_alignment: f32,
    pub top_alignment: f32,
    pub forward_approach_speed: f32,
    pub pitch_rate: f32,
    pub ball_speed_change: f32,
}
