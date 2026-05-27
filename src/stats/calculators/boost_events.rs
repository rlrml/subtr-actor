use super::*;

#[derive(Clone, Debug)]
pub(super) struct PendingBoostPickupEvent {
    pub(super) frame: usize,
    pub(super) time: f32,
    pub(super) player_id: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) pad_type: BoostPickupPadType,
    pub(super) field_half: BoostPickupFieldHalf,
    pub(super) activity: BoostPickupActivity,
    pub(super) boost_before: Option<f32>,
    pub(super) boost_after: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostPickupComparisonEvent {
    pub comparison: BoostPickupComparison,
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub pad_type: BoostPickupPadType,
    pub field_half: BoostPickupFieldHalf,
    pub activity: BoostPickupActivity,
    pub reported_frame: Option<usize>,
    pub reported_time: Option<f32>,
    pub inferred_frame: Option<usize>,
    pub inferred_time: Option<f32>,
    pub boost_before: Option<f32>,
    pub boost_after: Option<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostLedgerTransactionKind {
    Collected,
    Stolen,
    Overfill,
    Respawn,
    Used,
    UsedAllocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostLedgerEvent {
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub transaction: BoostLedgerTransactionKind,
    pub amount: f32,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<StatLabel>,
    pub boost_before: Option<f32>,
    pub boost_after: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostStateEvent {
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub boost_amount: f32,
    pub boost_before: Option<f32>,
}
