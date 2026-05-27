use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoPlayerStats {
    pub demos_inflicted: u32,
    pub demos_taken: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoTeamStats {
    pub demos_inflicted: u32,
}
