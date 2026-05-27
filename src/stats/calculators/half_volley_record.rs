use super::*;

impl HalfVolleyCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_volley = false;
            stats.time_since_last_half_volley = stats
                .last_half_volley_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_half_volley = stats
                .last_half_volley_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(super) fn record_half_volley(&mut self, frame: &FrameInfo, mut event: HalfVolleyEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.record_player_stats(frame, &event);
        self.record_team_stats(&event);

        self.current_last_half_volley_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn record_player_stats(&mut self, frame: &FrameInfo, event: &HalfVolleyEvent) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_speed += event.ball_speed;
        player_stats.fastest_ball_speed = player_stats.fastest_ball_speed.max(event.ball_speed);
        player_stats.last_half_volley_time = Some(event.time);
        player_stats.last_half_volley_frame = Some(event.frame);
        player_stats.time_since_last_half_volley = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_half_volley =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_team_stats(&mut self, event: &HalfVolleyEvent) {
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_speed += event.ball_speed;
        team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);
    }
}
