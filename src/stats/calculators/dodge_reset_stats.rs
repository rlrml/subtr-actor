use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetStats {
    pub count: u32,
    pub on_ball_count: u32,
}
