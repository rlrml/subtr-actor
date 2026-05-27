use super::*;

pub(crate) fn push_fifty_fifty_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    fifty_fifty: &[FiftyFiftyEvent],
) {
    for (index, event) in fifty_fifty.iter().enumerate() {
        let Some(winning_team_is_team_0) = event.winning_team_is_team_0 else {
            continue;
        };
        let Some(player_id) = (if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }) else {
            continue;
        };

        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "fifty_fifty:{}:{}:{}:{index}",
                    event.start_frame,
                    event.resolve_frame,
                    player_index(player_id)
                ),
                kind: SaMechanicKind::FiftyFifty,
                player_id: player_id.clone(),
                is_team_0: winning_team_is_team_0,
                frame_number: event.resolve_frame,
                time: event.resolve_time,
                confidence: 1.0,
            },
        );
    }
}
