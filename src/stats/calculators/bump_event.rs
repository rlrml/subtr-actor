use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub initiator: PlayerId,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    pub initiator_is_team_0: bool,
    pub victim_is_team_0: bool,
    pub is_team_bump: bool,
    pub strength: f32,
    pub confidence: f32,
    pub contact_distance: f32,
    pub closing_speed: f32,
    pub victim_impulse: f32,
    pub initiator_position: [f32; 3],
    pub victim_position: [f32; 3],
}
