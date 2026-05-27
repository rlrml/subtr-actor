use super::*;

impl MatchStatsCalculator {
    pub(super) fn record_goal_context_events(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        for goal_event in &events.goal_events {
            self.record_goal_context_event(players, goal_event, ball_position);
        }
    }

    fn record_goal_context_event(
        &mut self,
        players: &PlayerFrameState,
        goal_event: &GoalEvent,
        ball_position: Option<GoalContextPosition>,
    ) {
        let scoring_team_most_back_player =
            Self::most_back_player(players, goal_event.scoring_team_is_team_0);
        let defending_team_most_back_player =
            Self::most_back_player(players, !goal_event.scoring_team_is_team_0);
        let scorer_last_touch = goal_event
            .player
            .as_ref()
            .and_then(|player_id| self.last_touch_context_by_player.get(player_id))
            .filter(|touch| touch.is_team_0 == goal_event.scoring_team_is_team_0)
            .cloned();
        let ball_air_time_before_goal = self.ball_air_time_before_goal(goal_event.time);
        let goal_buildup =
            self.classify_goal_buildup(goal_event.time, goal_event.scoring_team_is_team_0);

        self.record_goal_context_stats(
            players,
            goal_event,
            scoring_team_most_back_player.as_ref(),
            defending_team_most_back_player.as_ref(),
        );

        self.goal_context_events.push(GoalContextEvent {
            time: goal_event.time,
            frame: goal_event.frame,
            scoring_team_is_team_0: goal_event.scoring_team_is_team_0,
            scorer: goal_event.player.clone(),
            scoring_team_most_back_player: scoring_team_most_back_player.clone(),
            defending_team_most_back_player: defending_team_most_back_player.clone(),
            ball_position,
            ball_air_time_before_goal,
            goal_buildup,
            scorer_last_touch,
            players: self.goal_player_contexts(
                players,
                goal_event.scoring_team_is_team_0,
                scoring_team_most_back_player.as_ref(),
                defending_team_most_back_player.as_ref(),
            ),
        });
    }
}
