use crate::*;
use serde::Serialize;

/// Column headers for the frame matrix emitted by [`NDArrayCollector`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NDArrayColumnHeaders {
    /// Column names emitted once per frame, independent of player ordering.
    pub global_headers: Vec<String>,
    /// Column names repeated once for each player in replay order.
    pub player_headers: Vec<String>,
}

impl NDArrayColumnHeaders {
    /// Builds a header set from global and per-player column names.
    pub fn new(global_headers: Vec<String>, player_headers: Vec<String>) -> Self {
        Self {
            global_headers,
            player_headers,
        }
    }
}

/// Replay metadata bundled with the ndarray column layout used to produce it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMetaWithHeaders {
    /// Replay metadata describing the teams and player ordering.
    pub replay_meta: ReplayMeta,
    /// Column headers associated with the emitted ndarray rows.
    pub column_headers: NDArrayColumnHeaders,
}

impl ReplayMetaWithHeaders {
    /// Flattens the global and per-player headers using a default player prefix.
    pub fn headers_vec(&self) -> Vec<String> {
        self.headers_vec_from(|_, _info, index| format!("Player {index} - "))
    }

    /// Flattens the global and per-player headers with a custom player prefix.
    pub fn headers_vec_from<F>(&self, player_prefix_getter: F) -> Vec<String>
    where
        F: Fn(&Self, &PlayerInfo, usize) -> String,
    {
        self.column_headers
            .global_headers
            .iter()
            .cloned()
            .chain(self.replay_meta.player_order().enumerate().flat_map(
                move |(player_index, info)| {
                    let player_prefix = player_prefix_getter(self, info, player_index);
                    self.column_headers
                        .player_headers
                        .iter()
                        .map(move |header| format!("{player_prefix}{header}"))
                },
            ))
            .collect()
    }
}
