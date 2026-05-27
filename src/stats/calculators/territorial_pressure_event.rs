use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TerritorialPressureEndReason {
    Relieved,
    Stoppage,
    BallMissing,
    ReplayEnd,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub team_is_team_0: bool,
    pub duration: f32,
    pub offensive_half_time: f32,
    pub offensive_third_time: f32,
    pub end_reason: TerritorialPressureEndReason,
}
