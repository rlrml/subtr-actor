use super::*;

pub(crate) fn goal_context_position(position: Option<subtr_actor::GoalContextPosition>) -> SaVec3 {
    position
        .map(|position| SaVec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        })
        .unwrap_or_default()
}

pub(crate) fn push_goal_context_events_from_timeline(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    goal_context: &[GoalContextEvent],
) {
    for (index, event) in goal_context.iter().enumerate() {
        let scorer = event.scorer.as_ref();
        let scoring_team_most_back_player = event.scoring_team_most_back_player.as_ref();
        let defending_team_most_back_player = event.defending_team_most_back_player.as_ref();
        push_pending_goal_context_event(
            pending_goal_context_events,
            emitted_goal_context_ids,
            format!("goal_context:{}:{}:{index}", event.frame, event.time),
            SaGoalContextEvent {
                frame_number: event.frame as u64,
                time: event.time,
                scoring_team_is_team_0: event.scoring_team_is_team_0 as u8,
                has_scorer: scorer.is_some() as u8,
                scorer_index: scorer.map(player_index).unwrap_or(0),
                has_scoring_team_most_back_player: scoring_team_most_back_player.is_some() as u8,
                scoring_team_most_back_player_index: scoring_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_defending_team_most_back_player: defending_team_most_back_player.is_some()
                    as u8,
                defending_team_most_back_player_index: defending_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_ball_position: event.ball_position.is_some() as u8,
                ball_position: goal_context_position(event.ball_position),
                has_ball_air_time_before_goal: event.ball_air_time_before_goal.is_some() as u8,
                ball_air_time_before_goal: event.ball_air_time_before_goal.unwrap_or(0.0),
                goal_buildup: goal_buildup_kind(event.goal_buildup),
            },
        );
    }
}
