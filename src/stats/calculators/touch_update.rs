use super::*;

impl TouchCalculator {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        touch_state: &TouchState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.reset_for_inactive_play(ball);
            return Ok(());
        }

        self.begin_sample(frame);
        self.apply_touch_events(
            frame,
            ball,
            players,
            vertical_state,
            &touch_state.touch_events,
        );
        self.credit_ball_movement(frame, ball, possession_state, fifty_fifty_state, live_play);
        self.previous_ball_velocity = ball.velocity();
        if let Some(player_id) = touch_state.last_touch_player.as_ref() {
            self.current_last_touch_player = Some(player_id.clone());
        }
        self.mark_current_last_touch();
        Ok(())
    }

    fn reset_for_inactive_play(&mut self, ball: &BallFrameState) {
        self.current_last_touch_player = None;
        self.previous_ball_velocity = ball.velocity();
        self.previous_ball_position = ball.position();
        self.pending_fifty_fifty_movement = None;
    }
}
