use super::*;

pub(crate) struct PendingGraphEvent {
    pub(crate) id: String,
    pub(crate) kind: SaMechanicKind,
    pub(crate) player_id: RemoteId,
    pub(crate) is_team_0: bool,
    pub(crate) frame_number: usize,
    pub(crate) time: f32,
    pub(crate) confidence: f32,
}

pub(crate) fn push_pending_graph_event(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: PendingGraphEvent,
) {
    if !emitted_mechanic_ids.insert(event.id) {
        return;
    }
    pending_events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: player_index(&event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

pub(crate) fn push_pending_team_event(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    id: String,
    event: SaTeamEvent,
) {
    if !emitted_team_event_ids.insert(id) {
        return;
    }
    pending_team_events.push(event);
}

pub(crate) fn push_pending_goal_context_event(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    id: String,
    event: SaGoalContextEvent,
) {
    if !emitted_goal_context_ids.insert(id) {
        return;
    }
    pending_goal_context_events.push(event);
}
