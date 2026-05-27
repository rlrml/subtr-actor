use super::*;

pub(crate) fn find_counter(counters: &[(RemoteId, i32)], player_id: &RemoteId) -> Option<i32> {
    counters
        .iter()
        .find_map(|(id, value)| (id == player_id).then_some(*value))
}

pub(crate) fn set_counter(counters: &mut Vec<(RemoteId, i32)>, player_id: RemoteId, value: i32) {
    if let Some((_, counter)) = counters.iter_mut().find(|(id, _)| id == &player_id) {
        *counter = value;
    } else {
        counters.push((player_id, value));
    }
}
