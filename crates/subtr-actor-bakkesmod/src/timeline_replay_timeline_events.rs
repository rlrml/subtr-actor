use super::*;

pub(crate) fn push_replay_timeline_event_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    timeline: &[TimelineEvent],
) {
    let mut occurrence_by_key = HashMap::new();
    for event in timeline {
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        let frame_number = event.frame.unwrap_or(0);
        let event_key = format!(
            "replay_timeline:{:?}:{}:{}:{}:{}",
            event.kind,
            event.time.to_bits(),
            frame_number,
            replay_player_index(index_map, player_id),
            is_team_0 as u8
        );
        let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
        let id = format!("{event_key}:{occurrence}");
        *occurrence += 1;
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id,
                kind: timeline_event_kind(event.kind),
                player_id: player_id.clone(),
                is_team_0,
                frame_number,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}
