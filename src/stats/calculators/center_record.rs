use super::*;

impl CenterCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_center = false;
            stats.time_since_last_center = stats
                .last_center_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_center = stats
                .last_center_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(super) fn record_center(&mut self, frame: &FrameInfo, event: CenterEvent) {
        self.record_player_stats(frame, &event);
        self.record_team_stats(&event);
        self.current_last_center_player = Some(event.player.clone());
        self.events.push(event);
        self.pending_touch = None;
    }

    fn record_player_stats(&mut self, frame: &FrameInfo, event: &CenterEvent) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_travel_distance += event.ball_travel_distance;
        player_stats.total_ball_advance_distance += event.ball_advance_distance;
        player_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        player_stats.longest_center_distance = player_stats
            .longest_center_distance
            .max(event.ball_travel_distance);
        player_stats.last_center_time = Some(event.time);
        player_stats.last_center_frame = Some(event.frame);
        player_stats.time_since_last_center = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_center =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_team_stats(&mut self, event: &CenterEvent) {
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_travel_distance += event.ball_travel_distance;
        team_stats.total_ball_advance_distance += event.ball_advance_distance;
        team_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        team_stats.longest_center_distance = team_stats
            .longest_center_distance
            .max(event.ball_travel_distance);
    }
}
