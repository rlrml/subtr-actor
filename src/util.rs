use crate::*;
use boxcars::{HeaderProp, RemoteId};
use serde::Serialize;

#[macro_export]
macro_rules! fmt_err {
    ($( $item:expr ),* $(,)?) => {
        Err(format!($( $item ),*))
    };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMeta {
    pub team_zero: Vec<PlayerInfo>,
    pub team_one: Vec<PlayerInfo>,
    pub all_headers: Vec<(String, HeaderProp)>,
}

impl ReplayMeta {
    pub fn player_count(&self) -> usize {
        self.team_one.len() + self.team_zero.len()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerInfo {
    pub remote_id: RemoteId,
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
    pub name: String,
}

pub fn find_player_stats(
    player_id: &RemoteId,
    name: &String,
    all_player_stats: &Vec<Vec<(String, HeaderProp)>>,
) -> Result<std::collections::HashMap<String, HeaderProp>, String> {
    Ok(all_player_stats
        .iter()
        .find(|player_stats| matches_stats(player_id, name, player_stats))
        .ok_or(format!(
            "Player not found {:?} {:?}",
            player_id, all_player_stats
        ))?
        .iter()
        .cloned()
        .collect())
}

fn matches_stats(player_id: &RemoteId, name: &String, props: &Vec<(String, HeaderProp)>) -> bool {
    if platform_matches(player_id, props) != Ok(true) {
        return false;
    }
    match player_id {
        RemoteId::Epic(_) => name_matches(name, props),
        RemoteId::Steam(id) => online_id_matches(*id, props),
        RemoteId::Xbox(id) => online_id_matches(*id, props),
        RemoteId::PlayStation(ps4id) => online_id_matches(ps4id.online_id, props),
        RemoteId::PsyNet(psynet_id) => online_id_matches(psynet_id.online_id, props),
        RemoteId::Switch(switch_id) => online_id_matches(switch_id.online_id, props),
        _ => false,
    }
}

fn name_matches(name: &String, props: &Vec<(String, HeaderProp)>) -> bool {
    if let Ok((_, HeaderProp::Str(stat_name))) = get_prop("Name", props) {
        *name == stat_name
    } else {
        false
    }
}

fn online_id_matches(id: u64, props: &Vec<(String, HeaderProp)>) -> bool {
    if let Ok((_, HeaderProp::QWord(props_id))) = get_prop("OnlineID", props) {
        id == props_id
    } else {
        false
    }
}

fn platform_matches(
    player_id: &RemoteId,
    props: &Vec<(String, HeaderProp)>,
) -> Result<bool, String> {
    if let (
        _,
        HeaderProp::Byte {
            kind: _,
            value: Some(value),
        },
    ) = get_prop("Platform", props)?
    {
        Ok(match (player_id, value.as_ref()) {
            (RemoteId::Steam(_), "OnlinePlatform_Steam") => true,
            (RemoteId::PlayStation(_), "OnlinePlatform_PS4") => true,
            (RemoteId::Epic(_), "OnlinePlatform_Epic") => true,
            (RemoteId::PsyNet(_), "OnlinePlatform_PS4") => true,
            (RemoteId::Xbox(_), "OnlinePlatform_Dingo") => true,
            // XXX: not sure if this is right.
            (RemoteId::Switch(_), "OnlinePlatform_Switch") => true,
            // TODO: There are still a few cases remaining.
            _ => false,
        })
    } else {
        fmt_err!("Unexpected platform value {:?}", props)
    }
}

fn get_prop(prop: &str, props: &Vec<(String, HeaderProp)>) -> Result<(String, HeaderProp), String> {
    props
        .iter()
        .find(|(attr, _)| attr == prop)
        .ok_or("Coudn't find name property".to_string())
        .cloned()
}
