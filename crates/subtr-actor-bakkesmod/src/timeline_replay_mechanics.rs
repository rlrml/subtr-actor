use super::*;

pub(crate) fn push_replay_mechanic_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    mechanics: &[MechanicEvent],
) {
    for event in mechanics {
        let Some(kind) = mechanic_kind(&event.kind) else {
            continue;
        };
        let (frame_number, time) = mechanic_start(event);
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: event.id.clone(),
                kind,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number,
                time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_replay_backboard_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    backboard: &[BackboardBounceEvent],
) {
    for (index, event) in backboard.iter().enumerate() {
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_backboard:{}:{}:{index}",
                    event.frame,
                    replay_player_index(index_map, &event.player)
                ),
                kind: SaMechanicKind::Backboard,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_replay_whiff_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    whiffs: &[WhiffEvent],
) {
    for (index, event) in whiffs.iter().enumerate() {
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_whiff:{}:{}:{index}",
                    event.frame,
                    replay_player_index(index_map, &event.player)
                ),
                kind: SaMechanicKind::Whiff,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}
