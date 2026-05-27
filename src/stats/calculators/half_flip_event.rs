use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfFlipEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub end_speed: f32,
    pub start_backward_alignment: f32,
    pub best_reorientation_alignment: f32,
    pub best_forward_reversal: f32,
    pub max_forward_vertical: f32,
    pub confidence: f32,
}
