use super::*;

impl WallAerialCalculator {
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
            self.reset_live_state(ball);
            return Ok(());
        }

        self.update_active_wall_control(
            frame,
            Self::control_observation(ball, players, touch_state),
        );
        self.update_wall_contacts_and_takeoffs(frame, players);
        self.prune_armed_aerials(frame.time);
        self.record_touch_events(frame, ball, players, touch_state);
        self.previous_ball_velocity = ball.velocity();
        self.mark_current_last_wall_aerial_player();
        Ok(())
    }

    fn reset_live_state(&mut self, ball: &BallFrameState) {
        self.active_wall_controls.clear();
        self.recent_wall_contacts.clear();
        self.armed_aerials.clear();
        self.recent_event_times.clear();
        self.previous_ball_velocity = ball.velocity();
        self.current_last_wall_aerial_player = None;
    }

    fn record_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);
        for touch in &touch_state.touch_events {
            if let Some(event) = self.controlled_play_event(ball, players, touch, ball_speed_change)
            {
                if let Some(armed) = self.armed_aerials.get_mut(&event.player) {
                    armed.recorded = true;
                }
                self.record_event(frame, event);
            }
        }
    }
}
