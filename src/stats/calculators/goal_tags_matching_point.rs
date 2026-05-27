use super::*;

pub(super) fn tag_goals_by_point_mechanic_event<E: GoalMechanicPointEvent>(
    goals: &[GoalContextEvent],
    events: &[E],
    kind: GoalTagKind,
    max_event_to_goal_seconds: f32,
) -> Vec<GoalTagEvent> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index, goal };
        let Some(event) = events
            .iter()
            .filter(|event| point_event_matches_goal(*event, goal))
            .filter(|event| goal.time - event.event_time() <= max_event_to_goal_seconds)
            .max_by(|left, right| {
                left.event_time()
                    .total_cmp(&right.event_time())
                    .then_with(|| left.event_frame().cmp(&right.event_frame()))
            })
        else {
            continue;
        };

        tags.push(goal_tag_with_modifiers(
            ctx,
            kind,
            event.event_confidence(),
            mechanic_goal_modifiers(goal, event.event_player()),
            mechanic_goal_evidence(goal, point_mechanic_evidence(event)),
        ));
    }
    tags
}

pub(super) fn point_event_matches_goal<E: GoalMechanicPointEvent>(
    event: &E,
    goal: &GoalContextEvent,
) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.event_team_is_team_0() == goal.scoring_team_is_team_0
        && event.event_time() <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.event_frame() <= goal.frame
}
