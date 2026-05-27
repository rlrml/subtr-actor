use super::*;

pub(crate) fn push_replay_boost_pickup_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    boost_pickups: &[BoostPickupComparisonEvent],
) {
    for (index, event) in boost_pickups.iter().enumerate() {
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_boost_pickup:{}:{}:{:?}:{:?}:{index}",
                    event.frame,
                    replay_player_index(index_map, &event.player_id),
                    event.reported_frame,
                    event.inferred_frame
                ),
                kind: SaMechanicKind::BoostPickup,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_replay_bump_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    bumps: &[BumpEvent],
) {
    for (index, event) in bumps.iter().enumerate() {
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_bump:{}:{}:{}:{index}",
                    event.frame,
                    replay_player_index(index_map, &event.initiator),
                    replay_player_index(index_map, &event.victim)
                ),
                kind: SaMechanicKind::Bump,
                player_id: event.initiator.clone(),
                is_team_0: event.initiator_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }
}
