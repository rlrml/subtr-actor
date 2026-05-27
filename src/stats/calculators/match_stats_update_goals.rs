use super::*;

impl MatchStatsCalculator {
    pub(super) fn record_player_goal_deltas(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        current_stats: &mut CorePlayerStats,
        previous_stats: &CorePlayerStats,
    ) {
        for _ in 0..(current_stats.goals - previous_stats.goals).max(0) {
            let pending_goal_event =
                self.take_pending_goal_event(&player.player_id, player.is_team_0);
            if let Some(pending_goal_event) = pending_goal_event.as_ref() {
                self.record_pending_goal_context(current_stats, pending_goal_event, player);
            }
            let goal_time = pending_goal_event
                .as_ref()
                .map(|event| event.event.time)
                .unwrap_or(frame.time);
            let goal_buildup = pending_goal_event
                .as_ref()
                .map(|event| event.goal_buildup)
                .unwrap_or_else(|| self.classify_goal_buildup(goal_time, player.is_team_0));
            let goal_frame = pending_goal_event
                .as_ref()
                .map(|event| event.event.frame)
                .unwrap_or(frame.frame_number);
            self.record_goal_timing(current_stats, pending_goal_event, goal_time);
            current_stats
                .scoring_context
                .goal_buildup
                .record(goal_buildup);
            self.timeline.push(TimelineEvent {
                time: goal_time,
                frame: Some(goal_frame),
                kind: TimelineEventKind::Goal,
                player_id: Some(player.player_id.clone()),
                is_team_0: Some(player.is_team_0),
            });
        }
    }

    fn record_pending_goal_context(
        &mut self,
        stats: &mut CorePlayerStats,
        pending_goal_event: &PendingGoalEvent,
        player: &PlayerSample,
    ) {
        let scorer_last_touch =
            self.reconcile_goal_context_scorer(&pending_goal_event.event, &player.player_id);
        if let Some(touch_position) = scorer_last_touch.and_then(|touch| touch.ball_position) {
            stats
                .scoring_context
                .record_scoring_goal_last_touch_position(touch_position);
        }
        if let Some(ball_air_time_before_goal) = pending_goal_event.ball_air_time_before_goal {
            stats
                .scoring_context
                .record_goal_ball_air_time(ball_air_time_before_goal);
        }
    }

    fn record_goal_timing(
        &self,
        stats: &mut CorePlayerStats,
        pending_goal_event: Option<PendingGoalEvent>,
        goal_time: f32,
    ) {
        let time_after_kickoff = pending_goal_event
            .and_then(|event| event.time_after_kickoff)
            .or_else(|| {
                self.active_kickoff_touch_time
                    .map(|kickoff_touch_time| (goal_time - kickoff_touch_time).max(0.0))
            });
        if let Some(time_after_kickoff) = time_after_kickoff {
            stats
                .scoring_context
                .goal_after_kickoff
                .record_goal(time_after_kickoff);
        }
    }
}
