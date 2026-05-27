use super::*;

impl MatchStatsCalculator {
    pub(super) fn update_last_touch_contexts(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        for touch in touch_events {
            let Some(player_id) = touch.player.clone() else {
                continue;
            };
            self.record_last_touch_context(players, touch, player_id, ball_position);
        }
    }

    fn record_last_touch_context(
        &mut self,
        players: &PlayerFrameState,
        touch: &TouchEvent,
        player_id: PlayerId,
        ball_position: Option<GoalContextPosition>,
    ) {
        let touch_team_most_back_player = Self::most_back_player(players, touch.team_is_team_0);
        let other_team_most_back_player = Self::most_back_player(players, !touch.team_is_team_0);
        let touch_players = self.goal_player_contexts(
            players,
            touch.team_is_team_0,
            touch_team_most_back_player.as_ref(),
            other_team_most_back_player.as_ref(),
        );
        self.last_touch_context_by_player.insert(
            player_id.clone(),
            GoalTouchContext {
                time: touch.time,
                frame: touch.frame,
                player: player_id.clone(),
                is_team_0: touch.team_is_team_0,
                ball_position,
                player_position: Self::player_position(players, &player_id)
                    .map(GoalContextPosition::from),
                players: touch_players,
            },
        );
    }

    pub(super) fn reconcile_goal_context_scorer(
        &mut self,
        goal_event: &GoalEvent,
        scorer: &PlayerId,
    ) -> Option<GoalTouchContext> {
        let scorer_last_touch = self
            .last_touch_context_by_player
            .get(scorer)
            .filter(|touch| touch.is_team_0 == goal_event.scoring_team_is_team_0)
            .cloned();
        if let Some(context) = self.goal_context_events.iter_mut().rev().find(|context| {
            context.frame == goal_event.frame
                && context.time == goal_event.time
                && context.scoring_team_is_team_0 == goal_event.scoring_team_is_team_0
                && context.scorer.as_ref() != Some(scorer)
        }) {
            context.scorer = Some(scorer.clone());
            context.scorer_last_touch = scorer_last_touch.clone();
        }
        scorer_last_touch
    }
}
