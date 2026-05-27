use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapPlayerStats {
    pub count: u32,
    pub is_last_double_tap: bool,
    pub last_double_tap_time: Option<f32>,
    pub last_double_tap_frame: Option<usize>,
    pub time_since_last_double_tap: Option<f32>,
    pub frames_since_last_double_tap: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapTeamStats {
    pub count: u32,
}
