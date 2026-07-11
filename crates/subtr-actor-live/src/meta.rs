use boxcars::RemoteId;
use serde::{Deserialize, Serialize};
use subtr_actor::{PlayerId, PlayerInfo, ReplayMeta, hitbox_family_for_body_id};

use crate::model::LivePlayerFrame;

/// Roster metadata for a live match, sufficient to build a
/// [`subtr_actor::ReplayMeta`] for the analysis graph.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveMatchMeta {
    pub players: Vec<LiveRosterPlayer>,
    /// Match-level context that cannot be derived from player frames.
    pub context: LiveMatchContext,
}

/// Match-level context supplied by the host (game plugin) out of band from
/// player frames: none of these fields is derivable from a [`LivePlayerFrame`].
///
/// Deliberately excluded from [`LiveMatchMeta::signature`]: the signature is
/// roster *identity*, and context changes broadcast their own roster update
/// instead of pretending the roster changed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LiveMatchContext {
    pub match_guid: Option<String>,
    pub playlist_id: Option<i32>,
    pub map_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveRosterPlayer {
    pub player_id: PlayerId,
    pub name: Option<String>,
    pub is_team_0: bool,
    pub car_body_id: Option<u32>,
}

impl LiveMatchMeta {
    pub fn from_player_frames(players: &[LivePlayerFrame]) -> Self {
        Self {
            players: players
                .iter()
                .map(|player| LiveRosterPlayer {
                    player_id: player.canonical_player_id(),
                    name: player.name.clone(),
                    is_team_0: player.is_team_0,
                    car_body_id: player.car_body_id,
                })
                .collect(),
            context: LiveMatchContext::default(),
        }
    }

    /// Identity of the roster; a change means the graph needs new replay meta.
    pub fn signature(&self) -> Vec<(PlayerId, bool, Option<String>)> {
        self.players
            .iter()
            .map(|player| {
                (
                    player.player_id.clone(),
                    player.is_team_0,
                    player.name.clone(),
                )
            })
            .collect()
    }

    pub fn replay_meta(&self) -> ReplayMeta {
        let mut team_zero = Vec::new();
        let mut team_one = Vec::new();
        for player in &self.players {
            let info = PlayerInfo {
                remote_id: player.player_id.clone(),
                stats: None,
                name: player
                    .name
                    .clone()
                    .unwrap_or_else(|| default_live_player_name(&player.player_id)),
                car_body_id: player.car_body_id,
                car_body_name: None,
                car_hitbox_family: player
                    .car_body_id
                    .and_then(hitbox_family_for_body_id)
                    .map(|family| format!("{family:?}"))
                    .or_else(|| Some("Octane".to_owned())),
                // Live frames don't carry replicated camera presets.
                camera_settings: None,
            };
            if player.is_team_0 {
                team_zero.push(info);
            } else {
                team_one.push(info);
            }
        }
        ReplayMeta {
            team_zero,
            team_one,
            game_type: Default::default(),
            season: None,
            all_headers: Vec::new(),
        }
    }
}

pub fn default_live_player_name(player_id: &PlayerId) -> String {
    match player_id {
        RemoteId::SplitScreen(index) => format!("Player {index}"),
        _ => format!("{player_id:?}"),
    }
}
