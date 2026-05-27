use super::*;

pub(super) fn pass_event_matches_goal(event: &PassEvent, goal: &GoalContextEvent) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.is_team_0 == goal.scoring_team_is_team_0
        && event.time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.frame <= goal.frame
        && goal.scorer.as_ref() == Some(&event.receiver)
        && goal
            .scorer_last_touch
            .as_ref()
            .is_some_and(|touch| touch.player == event.receiver && touch.frame == event.frame)
}
