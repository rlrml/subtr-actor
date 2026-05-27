use super::*;

impl TouchStateCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
    ) -> TouchState {
        let touch_events = if live_play_state.is_live_play {
            self.live_play_touch_events(frame, ball, players, events)
        } else {
            self.reset_live_play_state();
            Vec::new()
        };

        if let Some(last_touch) = touch_events.last() {
            self.current_last_touch = Some(last_touch.clone());
        }
        self.previous_ball_linear_velocity = Self::current_ball_linear_velocity(ball);
        self.previous_ball_angular_velocity = Self::current_ball_angular_velocity(ball);

        TouchState {
            touch_events,
            last_touch: self.current_last_touch.clone(),
            last_touch_player: self
                .current_last_touch
                .as_ref()
                .and_then(|touch| touch.player.clone()),
            last_touch_team_is_team_0: self
                .current_last_touch
                .as_ref()
                .map(|touch| touch.team_is_team_0),
        }
    }

    fn live_play_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> Vec<TouchEvent> {
        self.prune_recent_touch_candidates(frame.frame_number);
        self.update_recent_touch_candidates(frame, ball, players);
        self.confirmed_touch_events(frame, ball, players, events)
    }

    fn reset_live_play_state(&mut self) {
        self.current_last_touch = None;
        self.recent_touch_candidates.clear();
    }
}
