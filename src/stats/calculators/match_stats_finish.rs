use super::*;

impl MatchStatsCalculator {
    pub fn finish(&mut self) -> SubtrActorResult<()> {
        let pending_goal_events = std::mem::take(&mut self.pending_goal_events);
        for pending_goal_event in pending_goal_events {
            let Some(scorer) = pending_goal_event.event.player.clone() else {
                continue;
            };
            self.finish_pending_goal_event(pending_goal_event, scorer);
        }

        self.timeline.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(())
    }

    fn finish_pending_goal_event(
        &mut self,
        pending_goal_event: PendingGoalEvent,
        scorer: PlayerId,
    ) {
        let scorer_last_touch =
            self.reconcile_goal_context_scorer(&pending_goal_event.event, &scorer);
        let scorer_stats = self.player_stats.entry(scorer.clone()).or_default();
        scorer_stats.goals += 1;
        if let Some(touch_position) = scorer_last_touch.and_then(|touch| touch.ball_position) {
            scorer_stats
                .scoring_context
                .record_scoring_goal_last_touch_position(touch_position);
        }
        if let Some(time_after_kickoff) = pending_goal_event.time_after_kickoff {
            scorer_stats
                .scoring_context
                .goal_after_kickoff
                .record_goal(time_after_kickoff);
        }
        scorer_stats
            .scoring_context
            .goal_buildup
            .record(pending_goal_event.goal_buildup);
        if let Some(ball_air_time_before_goal) = pending_goal_event.ball_air_time_before_goal {
            scorer_stats
                .scoring_context
                .record_goal_ball_air_time(ball_air_time_before_goal);
        }

        self.timeline.push(TimelineEvent {
            time: pending_goal_event.event.time,
            frame: Some(pending_goal_event.event.frame),
            kind: TimelineEventKind::Goal,
            player_id: Some(scorer),
            is_team_0: Some(pending_goal_event.event.scoring_team_is_team_0),
        });
    }
}
