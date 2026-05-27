use serde::Serialize;
use std::collections::BTreeMap;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrameScaffold {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    #[ts(type = "Record<string, unknown>")]
    pub team_zero: BTreeMap<String, serde_json::Value>,
    #[ts(type = "Record<string, unknown>")]
    pub team_one: BTreeMap<String, serde_json::Value>,
    pub players: Vec<ReplayStatsPlayerIdentity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsPlayerIdentity {
    #[serde(rename = "player_id")]
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
}
