use super::*;

impl MatchStatsCalculator {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_frame_tracking(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        touch_state: &TouchState,
    ) {
        self.update_kickoff_reference(gameplay, events);
        self.prune_goal_buildup_samples(frame.time);
        self.update_ball_ground_contact(frame, ball);
        if live_play_state.is_live_play {
            self.record_goal_buildup_sample(frame, ball);
            self.record_goal_buildup_pressure_events(events);
            self.update_boost_leadup_samples(frame, players);
        } else if events.goal_events.is_empty() {
            self.last_touch_context_by_player.clear();
            self.boost_leadup_samples_by_player.clear();
            self.last_ball_ground_contact_time = None;
        }
        self.update_last_touch_contexts(ball, players, &touch_state.touch_events);
        self.record_goal_context_events(ball, players, events);
        self.queue_pending_goal_events(events);
    }

    fn queue_pending_goal_events(&mut self, events: &FrameEventsState) {
        let pending_goal_events: Vec<_> = events
            .goal_events
            .iter()
            .cloned()
            .map(|event| PendingGoalEvent {
                time_after_kickoff: self
                    .active_kickoff_touch_time
                    .map(|kickoff_touch_time| (event.time - kickoff_touch_time).max(0.0)),
                goal_buildup: self.classify_goal_buildup(event.time, event.scoring_team_is_team_0),
                ball_air_time_before_goal: self.ball_air_time_before_goal(event.time),
                event,
            })
            .collect();
        self.pending_goal_events.extend(pending_goal_events);
    }
}
