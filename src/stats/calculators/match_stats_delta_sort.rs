use super::*;

pub(super) fn player_id_sort_key(player_id: &PlayerId) -> String {
    match player_id {
        boxcars::RemoteId::PlayStation(id) => {
            format!("playstation:{}:{}:{:?}", id.online_id, id.name, id.unknown1)
        }
        boxcars::RemoteId::PsyNet(id) => format!("psynet:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::SplitScreen(id) => format!("splitscreen:{id}"),
        boxcars::RemoteId::Steam(id) => format!("steam:{id}"),
        boxcars::RemoteId::Switch(id) => format!("switch:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::Xbox(id) => format!("xbox:{id}"),
        boxcars::RemoteId::QQ(id) => format!("qq:{id}"),
        boxcars::RemoteId::Epic(id) => format!("epic:{id}"),
    }
}
