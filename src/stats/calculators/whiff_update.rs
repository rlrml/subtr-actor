use super::*;

impl WhiffCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.active_candidates.clear();
            self.current_last_whiff_player = None;
            return Ok(());
        }

        self.begin_sample(frame);
        self.finish_touched_candidates(frame, touch_state);
        if touch_state.touch_events.is_empty() {
            if let Some(ball_position) = ball.position() {
                self.update_active_candidates(
                    frame,
                    ball_position,
                    ball.velocity().unwrap_or(glam::Vec3::ZERO),
                    players,
                );
            }
        }

        if let Some(player_id) = self.current_last_whiff_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_whiff = true;
            }
        }

        Ok(())
    }
}
