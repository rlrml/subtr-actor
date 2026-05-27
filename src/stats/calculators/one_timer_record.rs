use super::*;

impl OneTimerCalculator {
    pub(super) fn record_one_timer(&mut self, frame: &FrameInfo, event: OneTimerEvent) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_speed += event.ball_speed;
        player_stats.fastest_ball_speed = player_stats.fastest_ball_speed.max(event.ball_speed);
        player_stats.total_pass_distance += event.pass_travel_distance;
        player_stats.last_one_timer_time = Some(event.time);
        player_stats.last_one_timer_frame = Some(event.frame);
        player_stats.time_since_last_one_timer = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_one_timer =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_speed += event.ball_speed;
        team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);

        self.current_last_one_timer_player = Some(event.player.clone());
        self.events.push(event);
    }
}
