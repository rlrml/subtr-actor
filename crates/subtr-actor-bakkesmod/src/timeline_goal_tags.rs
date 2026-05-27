use super::*;

pub(crate) fn push_goal_tag_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    goal_tags: &[GoalTagEvent],
) {
    for event in goal_tags {
        let Some(scorer) = event.scorer.as_ref() else {
            continue;
        };
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "goal_tag:{}:{}:{:?}:{}",
                    event.goal_index,
                    event.frame,
                    event.kind,
                    player_index(scorer)
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
