use super::*;

impl FlickCalculator {
    pub(super) fn apply_event(&mut self, frame: &FrameInfo, mut event: FlickEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_flick = true;
        stats.time_since_last_flick = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_flick = Some(frame.frame_number.saturating_sub(event.frame));

        self.current_last_flick_player = Some(event.player.clone());
        self.events.push(event);
    }

    pub(super) fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let ball_impulse = Self::ball_impulse(frame, ball, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let Some(player) = players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
            else {
                continue;
            };
            let Some(dodge_start) = self.recent_dodge_starts.get(player_id) else {
                continue;
            };
            let Some(event) =
                self.candidate_event(ball, player, touch_event, dodge_start, ball_impulse)
            else {
                continue;
            };

            self.apply_event(frame, event);
        }

        if let Some(player_id) = self.current_last_flick_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_flick = true;
            }
        }
    }
}
