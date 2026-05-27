use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RoleState {
    #[default]
    Unknown,
    FirstMan,
    SecondMan,
    ThirdMan,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PlayDepthState {
    #[default]
    Unknown,
    BehindPlay,
    LevelWithPlay,
    AheadOfPlay,
}
