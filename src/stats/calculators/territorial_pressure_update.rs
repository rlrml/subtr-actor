use super::*;

impl TerritorialPressureCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.last_frame = Some(frame.into());
        if !live_play_state.is_live_play {
            self.candidate = None;
            self.end_active_session(frame, TerritorialPressureEndReason::Stoppage);
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.candidate = None;
            self.end_active_session(frame, TerritorialPressureEndReason::BallMissing);
            return Ok(());
        };

        self.stats.tracked_time += frame.dt;
        if self.active.is_some() {
            self.update_active_session(frame, ball.position().y, possession_state);
        } else {
            self.update_candidate(frame, ball.position().y);
        }
        Ok(())
    }
}
