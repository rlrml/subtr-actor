use std::collections::HashMap;

use serde_json::Value;
use subtr_actor::{PlayerId, PlayerInfo, ReplayMeta};

#[derive(Clone)]
pub(crate) struct PlayerDisplay {
    pub(crate) name: String,
    pub(crate) team: &'static str,
}

pub(crate) fn player_id_string(player_id: &PlayerId) -> String {
    match serde_json::to_value(player_id) {
        Ok(Value::Object(map)) if map.len() == 1 => {
            let (kind, value) = map.into_iter().next().expect("map has one value");
            match value {
                Value::String(value) => format!("{kind}:{value}"),
                other => format!("{kind}:{other}"),
            }
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
    }
}

pub(crate) fn player_display_map(meta: &ReplayMeta) -> HashMap<String, PlayerDisplay> {
    meta.team_zero
        .iter()
        .map(|player| (player, "blue"))
        .chain(meta.team_one.iter().map(|player| (player, "orange")))
        .map(|(player, team)| {
            (
                player_id_string(&player.remote_id),
                player_display(player, team),
            )
        })
        .collect()
}

pub(crate) fn player_team_label(is_team_0: bool) -> &'static str {
    if is_team_0 {
        "blue"
    } else {
        "orange"
    }
}

fn player_display(player: &PlayerInfo, team: &'static str) -> PlayerDisplay {
    PlayerDisplay {
        name: player.name.clone(),
        team,
    }
}
