use super::*;

pub(super) fn tag_goals_by_air_dribble_event(
    goals: &[GoalContextEvent],
    events: &[BallCarryEvent],
    max_end_to_goal_seconds: f32,
) -> Vec<GoalTagEvent> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index, goal };
        let Some(event) = events
            .iter()
            .filter(|event| air_dribble_event_matches_goal(event, goal))
            .filter(|event| goal.time - event.end_time <= max_end_to_goal_seconds)
            .max_by(|left, right| {
                left.end_time
                    .total_cmp(&right.end_time)
                    .then_with(|| left.end_frame.cmp(&right.end_frame))
            })
        else {
            continue;
        };

        tags.push(goal_tag_with_modifiers(
            ctx,
            GoalTagKind::AirDribbleGoal,
            1.0,
            mechanic_goal_modifiers(goal, &event.player_id),
            mechanic_goal_evidence(goal, air_dribble_evidence(event)),
        ));
    }
    tags
}

pub(super) fn air_dribble_event_matches_goal(
    event: &BallCarryEvent,
    goal: &GoalContextEvent,
) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.kind == BallCarryKind::AirDribble
        && event.is_team_0 == goal.scoring_team_is_team_0
        && event.start_time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.end_time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.end_frame <= goal.frame
}
