use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SpeedFlipEvent {
    pub time: f32,
    pub frame: usize,
    pub resolved_time: f32,
    pub resolved_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub time_since_kickoff_start: f32,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub max_speed: f32,
    pub best_alignment: f32,
    pub diagonal_score: f32,
    pub cancel_score: f32,
    pub speed_score: f32,
    pub confidence: f32,
}
