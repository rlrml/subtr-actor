use super::*;

pub(crate) fn push_timeline_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    timeline: &[TimelineEvent],
) {
    let mut occurrence_by_key = HashMap::new();
    for event in timeline {
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        let frame_number = event.frame.unwrap_or(0);
        let event_key = format!(
            "{:?}:{}:{}:{}:{}",
            event.kind,
            event.time.to_bits(),
            frame_number,
            player_index(player_id),
            is_team_0 as u8
        );
        let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
        let id = format!("timeline:{event_key}:{occurrence}");
        *occurrence += 1;
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
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
