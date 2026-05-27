use super::*;

impl DoubleTapCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        backboard_bounce_state: &BackboardBounceState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.pending_backboard_bounces.clear();
        }

        self.prune_pending_backboard_bounces(frame.time);
        self.record_backboard_bounces(backboard_bounce_state);
        self.resolve_double_tap_touches(frame, ball, &touch_state.touch_events);

        if let Some(player_id) = self.current_last_double_tap_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_double_tap = true;
            }
        }
        Ok(())
    }
}
