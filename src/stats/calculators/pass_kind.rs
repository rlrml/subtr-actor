use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PassKind {
    Direct,
    Backboard,
    FiftyFifty,
    FiftyFiftyBackboard,
}

impl PassCalculator {
    pub(super) fn pass_kind(from_fifty_fifty: bool, went_off_backboard: bool) -> PassKind {
        match (from_fifty_fifty, went_off_backboard) {
            (true, true) => PassKind::FiftyFiftyBackboard,
            (true, false) => PassKind::FiftyFifty,
            (false, true) => PassKind::Backboard,
            (false, false) => PassKind::Direct,
        }
    }
}
