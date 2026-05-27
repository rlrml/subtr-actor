use super::*;

impl CenterCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        frame_events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.pending_touch = None;
            self.current_last_center_player = None;
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            return Ok(());
        };

        self.clear_disqualified_pending_center(frame_events);
        self.update_pending_center(frame, ball_position);
        self.update_touches(touch_state, frame_events, ball_position);
        self.mark_current_last_center();

        Ok(())
    }

    fn update_pending_center(&mut self, frame: &FrameInfo, ball_position: glam::Vec3) {
        let Some(pending) = self.pending_touch.as_ref() else {
            return;
        };
        if frame.time - pending.time > CENTER_MAX_DURATION_SECONDS {
            self.pending_touch = None;
            return;
        }

        if let Some(event) = Self::center_event_for_position(pending, frame, ball_position) {
            self.record_center(frame, event);
        }
    }

    fn update_touches(
        &mut self,
        touch_state: &TouchState,
        frame_events: &FrameEventsState,
        ball_position: glam::Vec3,
    ) {
        for touch in &touch_state.touch_events {
            self.update_touch(touch, frame_events, ball_position);
        }
    }

    fn update_touch(
        &mut self,
        touch: &TouchEvent,
        frame_events: &FrameEventsState,
        ball_position: glam::Vec3,
    ) {
        let Some(player) = touch.player.clone() else {
            self.pending_touch = None;
            return;
        };

        if Self::player_has_disqualifying_event(frame_events, &player, touch.team_is_team_0) {
            self.pending_touch = None;
            return;
        }

        self.pending_touch = Some(PendingCenterTouch {
            player,
            is_team_0: touch.team_is_team_0,
            time: touch.time,
            frame: touch.frame,
            ball_position,
        });
    }

    fn mark_current_last_center(&mut self) {
        if let Some(player_id) = self.current_last_center_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_center = true;
            }
        }
    }
}
