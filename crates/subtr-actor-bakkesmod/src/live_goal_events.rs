use super::*;

impl SaLiveEventGenerator {
    pub(crate) fn explicit_goal_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaGoalEvent],
    ) -> Vec<GoalEvent> {
        let mut goal_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let goal_event = GoalEvent {
                time,
                frame: frame_number,
                scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
                player: (event.has_player != 0).then_some(player_id(event.player_index)),
                team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
                team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
            };
            if self
                .last_goal_event
                .as_ref()
                .is_some_and(|previous| goal_event_is_duplicate(previous, &goal_event))
            {
                continue;
            }
            self.last_goal_event = Some(goal_event.clone());
            goal_events.push(goal_event);
        }
        goal_events
    }
}
