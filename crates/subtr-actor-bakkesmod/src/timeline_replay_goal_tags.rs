use super::*;

pub(crate) fn push_replay_goal_tag_annotations(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    goal_tags: &[GoalTagEvent],
) {
    for event in goal_tags {
        let Some(scorer) = event.scorer.as_ref() else {
            continue;
        };
        push_replay_annotation(
            events,
            emitted_ids,
            index_map,
            PendingGraphEvent {
                id: format!(
                    "replay_goal_tag:{}:{}:{:?}:{}",
                    event.goal_index,
                    event.frame,
                    event.kind,
                    replay_player_index(index_map, scorer)
                ),
                kind: goal_tag_kind(event.kind),
                player_id: scorer.clone(),
                is_team_0: event.scoring_team_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }
}
