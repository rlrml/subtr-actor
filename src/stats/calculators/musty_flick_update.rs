use super::*;

impl MustyFlickCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_musty = false;
            stats.time_since_last_musty = stats
                .last_musty_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_musty = stats
                .last_musty_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(super) fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);

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
            let Some(dodge_start) = self.recent_dodge_starts.get(player_id).copied() else {
                continue;
            };
            let Some(mut event) =
                self.musty_candidate(ball, player, touch_event, dodge_start, ball_speed_change)
            else {
                continue;
            };
            event.sample_time = frame.time;
            event.sample_frame = frame.frame_number;
            self.record_touch_event(frame, player_id, event);
        }

        if let Some(player_id) = self.current_last_musty_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_musty = true;
            }
        }
    }

    pub(super) fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.current_last_musty_player = None;
        self.recent_dodge_starts.clear();
        self.previous_dodge_active.clear();
        self.previous_ball_velocity = ball.velocity();
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.reset_live_play_state(ball);
            return Ok(());
        }

        self.begin_sample(frame);
        self.prune_recent_dodge_starts(frame.time);
        self.track_dodge_starts(frame, players);
        self.apply_touch_events(frame, ball, players, touch_events);
        self.previous_ball_velocity = ball.velocity();
        Ok(())
    }
}
