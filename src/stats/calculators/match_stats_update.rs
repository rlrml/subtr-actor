use super::*;

impl MatchStatsCalculator {
    #[allow(clippy::too_many_arguments)]
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        touch_state: &TouchState,
    ) -> SubtrActorResult<()> {
        self.update_frame_tracking(
            frame,
            gameplay,
            ball,
            players,
            events,
            live_play_state,
            touch_state,
        );

        let processor_event_counts = self.record_processor_stat_events(events);
        self.update_player_core_stats(frame, players, &processor_event_counts);
        self.update_team_score_contexts(gameplay, players);
        self.sort_timeline();
        self.emit_core_stats_events(frame);

        Ok(())
    }
}
