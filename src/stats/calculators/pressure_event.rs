use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PressureEvent {
    pub time: f32,
    pub frame: usize,
    pub active: bool,
    pub field_half: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PressureEventState {
    pub(super) active: bool,
    pub(super) field_half: PressureHalfLabel,
}
