use super::*;

pub(crate) fn push_drainable_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    events: &ReplayStatsTimelineEvents,
) {
    push_mechanic_events_from_timeline(pending_events, emitted_mechanic_ids, &events.mechanics);
    push_backboard_events_from_timeline(pending_events, emitted_mechanic_ids, &events.backboard);
    push_whiff_events_from_timeline(pending_events, emitted_mechanic_ids, &events.whiff);
    push_boost_pickup_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.boost_pickups,
    );
    push_bump_events_from_timeline(pending_events, emitted_mechanic_ids, &events.bump);
    push_core_player_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.core_player,
    );
    push_timeline_events_from_timeline(pending_events, emitted_mechanic_ids, &events.timeline);
    push_fifty_fifty_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        &events.fifty_fifty,
    );
    push_goal_tag_events_from_timeline(pending_events, emitted_mechanic_ids, &events.goal_tags);
    push_rush_events_from_timeline(pending_team_events, emitted_team_event_ids, &events.rush);
    push_goal_context_events_from_timeline(
        pending_goal_context_events,
        emitted_goal_context_ids,
        &events.goal_context,
    );
    pending_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
    pending_team_events.sort_by(|left, right| {
        left.end_time
            .total_cmp(&right.end_time)
            .then_with(|| left.end_frame.cmp(&right.end_frame))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.is_team_0.cmp(&right.is_team_0))
    });
    pending_goal_context_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| {
                left.scoring_team_is_team_0
                    .cmp(&right.scoring_team_is_team_0)
            })
    });
}
