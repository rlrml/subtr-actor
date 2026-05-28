use serde::Serialize;

use super::PlayerId;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchEvent {
    pub time: f32,
    pub frame: usize,
    pub team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    pub closest_approach_distance: Option<f32>,
    pub dodge_contact: bool,
}
