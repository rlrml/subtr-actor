use super::*;

pub(crate) fn push_rush_events_from_timeline(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    rush: &[RushEvent],
) {
    for event in rush {
        push_pending_team_event(
            pending_team_events,
            emitted_team_event_ids,
            format!(
                "rush:{}:{}:{}",
                event.start_frame, event.end_frame, event.is_team_0
            ),
            SaTeamEvent {
                kind: SaTeamEventKind::Rush,
                is_team_0: event.is_team_0 as u8,
                start_frame: event.start_frame as u64,
                end_frame: event.end_frame as u64,
                start_time: event.start_time,
                end_time: event.end_time,
                attackers: event.attackers as u32,
                defenders: event.defenders as u32,
                confidence: 1.0,
            },
        );
    }
}
