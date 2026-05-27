use serde::Serialize;

use crate::PlayerId;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum MechanicTiming {
    Moment {
        frame: usize,
        time: f32,
    },
    Span {
        start_frame: usize,
        end_frame: usize,
        start_time: f32,
        end_time: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum MechanicEventPropertyValue {
    Text(String),
    Unsigned(u32),
    Float(f32),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MechanicEventProperty {
    pub key: String,
    pub value: MechanicEventPropertyValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MechanicEvent {
    pub id: String,
    pub kind: String,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub timing: MechanicTiming,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<MechanicEventProperty>,
}
