use super::*;

impl FlickCalculator {
    pub(super) fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.current_last_flick_player = None;
        self.active_setups.clear();
        self.recent_setups.clear();
        self.recent_dodge_starts.clear();
        self.previous_dodge_active.clear();
        self.previous_ball_velocity = ball.velocity();
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if !live_play_state.is_live_play {
            self.reset_live_play_state(ball);
            return Ok(());
        }

        self.begin_sample(frame);
        self.prune_recent_state(frame.time);
        self.update_control_setups(
            frame,
            ball,
            players,
            &touch_state.touch_events,
            touch_state.last_touch_player.as_ref(),
        );
        self.track_dodge_starts(frame, players);
        self.apply_touch_events(frame, ball, players, &touch_state.touch_events);
        self.previous_ball_velocity = ball.velocity();
        Ok(())
    }
}
