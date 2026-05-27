use super::*;

impl PassCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        backboard_bounce_state: &BackboardBounceState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.last_touch = None;
            self.current_last_completed_pass_player = None;
            self.emit_last_completed_event(frame, None);
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            self.emit_last_completed_event(frame, None);
            return Ok(());
        };

        for touch in &touch_state.touch_events {
            self.update_touch(
                frame,
                touch,
                ball_position,
                backboard_bounce_state,
                fifty_fifty_state,
            );
        }

        self.mark_current_last_completed_pass();
        self.emit_last_completed_event(frame, self.current_last_completed_pass_player.clone());

        Ok(())
    }

    fn update_touch(
        &mut self,
        frame: &FrameInfo,
        touch: &TouchEvent,
        ball_position: glam::Vec3,
        backboard_bounce_state: &BackboardBounceState,
        fifty_fifty_state: &FiftyFiftyState,
    ) {
        let Some(player) = touch.player.clone() else {
            self.last_touch = None;
            return;
        };

        if let Some(pass_event) =
            self.pass_event_for_touch(touch, &player, ball_position, backboard_bounce_state)
        {
            self.record_pass(frame, pass_event);
        }

        self.last_touch = Some(PendingPassTouch {
            player,
            is_team_0: touch.team_is_team_0,
            time: touch.time,
            frame: touch.frame,
            ball_position,
            from_fifty_fifty: Self::touch_from_fifty_fifty(touch, fifty_fifty_state),
        });
    }

    fn mark_current_last_completed_pass(&mut self) {
        if let Some(player_id) = self.current_last_completed_pass_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_completed_pass = true;
            }
        }
    }
}
