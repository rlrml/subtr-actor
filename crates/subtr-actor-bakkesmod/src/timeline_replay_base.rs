use super::*;

pub(crate) fn replay_player_index_map(replay_meta: &ReplayMeta) -> HashMap<RemoteId, u32> {
    replay_meta
        .player_order()
        .enumerate()
        .map(|(index, player)| (player.remote_id.clone(), index as u32))
        .collect()
}

pub(crate) fn replay_player_index(index_map: &HashMap<RemoteId, u32>, id: &RemoteId) -> u32 {
    index_map
        .get(id)
        .copied()
        .unwrap_or_else(|| player_index(id))
}

pub(crate) fn push_replay_annotation(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    event: PendingGraphEvent,
) {
    if !emitted_ids.insert(event.id) {
        return;
    }
    events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: replay_player_index(index_map, &event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

pub(crate) fn sort_replay_annotations(events: &mut [SaMechanicEvent]) {
    events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
}
