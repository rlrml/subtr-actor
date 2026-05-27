use super::*;

pub(crate) fn push_repeated_core_player_stat_events(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: &CorePlayerStatsEvent,
    kind: SaMechanicKind,
    count: i32,
) {
    for index in 0..count.max(0) {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "core_player:{:?}:{}:{}:{}",
                    kind,
                    event.frame,
                    player_index(&event.player),
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

pub(crate) fn push_core_player_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    core_player: &[CorePlayerStatsEvent],
) {
    for event in core_player {
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Shot,
            event.delta.shots,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Save,
            event.delta.saves,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Assist,
            event.delta.assists,
        );
    }
}
