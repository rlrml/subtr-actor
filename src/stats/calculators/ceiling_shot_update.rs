use super::*;

impl CeilingShotCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_ceiling_shot = false;
            stats.time_since_last_ceiling_shot = stats
                .last_ceiling_shot_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_ceiling_shot = stats
                .last_ceiling_shot_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
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
            let Some(recent_contact) = self.recent_ceiling_contacts.get(player_id).copied() else {
                continue;
            };
            let Some(event) =
                self.candidate_event(ball, player, touch_event, recent_contact, ball_speed_change)
            else {
                continue;
            };

            self.record_touch_event(frame, player_id, event);
        }

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
        }
    }

    pub(super) fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.current_last_ceiling_shot_player = None;
        self.recent_ceiling_contacts.clear();
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
        self.prune_recent_ceiling_contacts(frame.time);
        self.apply_touch_events(frame, ball, players, touch_events);
        self.update_recent_ceiling_contacts(frame, players);
        self.previous_ball_velocity = ball.velocity();
        Ok(())
    }
}
