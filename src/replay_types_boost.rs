use serde::Serialize;

use super::PlayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum BoostPadEventKind {
    PickedUp { sequence: u8 },
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum BoostPadSize {
    Big,
    Small,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostPadEvent {
    pub time: f32,
    pub frame: usize,
    pub pad_id: String,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    pub kind: BoostPadEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ResolvedBoostPad {
    pub index: usize,
    pub pad_id: Option<String>,
    pub size: BoostPadSize,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub position: boxcars::Vector3f,
}
