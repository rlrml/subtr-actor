use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardPlayerStats {
    pub count: u32,
    pub is_last_backboard: bool,
    pub last_backboard_time: Option<f32>,
    pub last_backboard_frame: Option<usize>,
    pub time_since_last_backboard: Option<f32>,
    pub frames_since_last_backboard: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardTeamStats {
    pub count: u32,
}
