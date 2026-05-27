use super::*;

impl PassCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_completed_pass = false;
            stats.time_since_last_completed_pass = stats
                .last_completed_pass_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_completed_pass = stats
                .last_completed_pass_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(super) fn record_pass(&mut self, frame: &FrameInfo, mut event: PassEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.record_passer_stats(frame, &event);
        self.player_stats
            .entry(event.receiver.clone())
            .or_default()
            .received_pass_count += 1;
        self.record_team_stats(&event);

        self.current_last_completed_pass_player = Some(event.passer.clone());
        self.events.push(event);
    }

    fn record_passer_stats(&mut self, frame: &FrameInfo, event: &PassEvent) {
        let passer_stats = self.player_stats.entry(event.passer.clone()).or_default();
        passer_stats.completed_pass_count += 1;
        passer_stats.total_pass_distance += event.ball_travel_distance;
        passer_stats.total_pass_advance += event.ball_advance_distance;
        passer_stats.longest_pass_distance = passer_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        passer_stats.last_completed_pass_time = Some(event.time);
        passer_stats.last_completed_pass_frame = Some(event.frame);
        passer_stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        passer_stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_team_stats(&mut self, event: &PassEvent) {
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.completed_pass_count += 1;
        team_stats.total_pass_distance += event.ball_travel_distance;
        team_stats.total_pass_advance += event.ball_advance_distance;
        team_stats.longest_pass_distance = team_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
    }

    pub(super) fn emit_last_completed_event(
        &mut self,
        frame: &FrameInfo,
        player: Option<PlayerId>,
    ) {
        if self.emitted_last_completed_pass_player == player {
            return;
        }
        self.emitted_last_completed_pass_player = player.clone();
        self.last_completed_events.push(PassLastCompletedEvent {
            time: frame.time,
            frame: frame.frame_number,
            player,
        });
    }
}
