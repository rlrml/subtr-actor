use serde::Serialize;

use super::PlayerId;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
}
