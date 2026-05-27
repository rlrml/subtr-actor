use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CeilingShotEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub ceiling_contact_time: f32,
    pub ceiling_contact_frame: usize,
    pub time_since_ceiling_contact: f32,
    pub ceiling_contact_position: [f32; 3],
    pub touch_position: [f32; 3],
    pub local_ball_position: [f32; 3],
    pub separation_from_ceiling: f32,
    pub roof_alignment: f32,
    pub forward_alignment: f32,
    pub forward_approach_speed: f32,
    pub ball_speed_change: f32,
    pub confidence: f32,
}
