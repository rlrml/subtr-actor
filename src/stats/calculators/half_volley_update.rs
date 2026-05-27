use super::*;

impl HalfVolleyCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.reset_live_play_state(ball);
            return Ok(());
        }

        self.update_player_movement_state(frame, players);
        self.update_floor_bounce(frame, ball, touch_state);
        self.apply_touches(frame, ball, touch_state);
        self.previous_ball_velocity = ball.velocity();
        self.mark_current_last_half_volley();

        Ok(())
    }

    fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.last_floor_bounce = None;
        self.last_ground_contacts.clear();
        self.recent_dodge_starts.clear();
        self.previous_dodge_active.clear();
        self.previous_ball_velocity = ball.velocity();
        self.current_last_half_volley_player = None;
    }

    fn update_floor_bounce(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
    ) {
        if let Some(bounce) = Self::detect_floor_bounce(
            frame,
            ball.sample(),
            self.previous_ball_velocity,
            &touch_state.touch_events,
        ) {
            self.last_floor_bounce = Some(bounce);
        }
    }

    fn apply_touches(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
    ) {
        for touch in &touch_state.touch_events {
            if let Some(event) = self.event_for_touch(ball, touch) {
                self.record_half_volley(frame, event);
            }
        }
    }

    fn mark_current_last_half_volley(&mut self) {
        if let Some(player_id) = self.current_last_half_volley_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_half_volley = true;
            }
        }
    }
}
