use super::*;

impl ReplayProcessor<'_> {
    pub(crate) fn goal_event_is_duplicate(
        &self,
        frame_time: f32,
        scoring_team_is_team_0: bool,
        team_zero_score: Option<i32>,
        team_one_score: Option<i32>,
    ) -> bool {
        const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;

        self.goal_events
            .last()
            .map(|event| {
                match (
                    team_zero_score,
                    team_one_score,
                    event.team_zero_score,
                    event.team_one_score,
                ) {
                    (
                        Some(team_zero),
                        Some(team_one),
                        Some(prev_team_zero),
                        Some(prev_team_one),
                    ) => team_zero == prev_team_zero && team_one == prev_team_one,
                    _ => {
                        event.scoring_team_is_team_0 == scoring_team_is_team_0
                            && (frame_time - event.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
                    }
                }
            })
            .unwrap_or(false)
    }

    pub(crate) fn derived_goal_score_tuple(
        &self,
        scoring_team_is_team_0: bool,
    ) -> (Option<i32>, Option<i32>) {
        let (mut team_zero_goals, mut team_one_goals) = self.last_known_goal_score_tuple();
        if scoring_team_is_team_0 {
            team_zero_goals += 1;
        } else {
            team_one_goals += 1;
        }
        (Some(team_zero_goals), Some(team_one_goals))
    }

    pub(crate) fn last_known_goal_score_tuple(&self) -> (i32, i32) {
        self.goal_events
            .last()
            .and_then(|event| event.team_zero_score.zip(event.team_one_score))
            .unwrap_or((0, 0))
    }
}
