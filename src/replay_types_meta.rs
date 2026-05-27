use boxcars::{HeaderProp, RemoteId};
use serde::Serialize;

/// [`ReplayMeta`] struct represents metadata about the replay being processed.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayMeta {
    pub team_zero: Vec<PlayerInfo>,
    pub team_one: Vec<PlayerInfo>,
    #[ts(as = "Vec<(String, crate::ts_bindings::HeaderPropTs)>")]
    pub all_headers: Vec<(String, HeaderProp)>,
}

impl ReplayMeta {
    /// Returns the total number of players involved in the game.
    pub fn player_count(&self) -> usize {
        self.team_one.len() + self.team_zero.len()
    }

    /// Returns an iterator over the [`PlayerInfo`] instances representing the players,
    /// in the order they are listed in the replay file.
    pub fn player_order(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.team_zero.iter().chain(self.team_one.iter())
    }
}

/// [`PlayerInfo`] struct provides detailed information about a specific player in the replay.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerInfo {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub remote_id: RemoteId,
    #[ts(as = "Option<std::collections::HashMap<String, crate::ts_bindings::HeaderPropTs>>")]
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
    pub name: String,
}
