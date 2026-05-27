use super::*;

pub(crate) fn push_replay_core_player_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    core_player: &[CorePlayerStatsEvent],
) {
    for event in core_player {
        for (kind, count) in [
            (SaMechanicKind::Shot, event.delta.shots),
            (SaMechanicKind::Save, event.delta.saves),
            (SaMechanicKind::Assist, event.delta.assists),
        ] {
            for index in 0..count.max(0) {
                push_replay_annotation(
                    events,
                    emitted_ids,
                    index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_core_player:{:?}:{}:{}:{}",
                            kind,
                            event.frame,
                            replay_player_index(index_map, &event.player),
                            index
                        ),
                        kind,
                        player_id: event.player.clone(),
                        is_team_0: event.is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
        }
    }
}
