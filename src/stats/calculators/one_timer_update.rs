use super::*;

impl OneTimerCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        pass_calculator: &PassCalculator,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.current_last_one_timer_player = None;
            self.processed_pass_events = pass_calculator.events().len();
            return Ok(());
        }

        for pass in &pass_calculator.events()[self.processed_pass_events..] {
            if pass.frame != frame.frame_number {
                continue;
            }
            if let Some(event) = Self::one_timer_event_for_pass(pass, ball) {
                self.record_one_timer(frame, event);
            }
        }
        self.processed_pass_events = pass_calculator.events().len();

        if let Some(player_id) = self.current_last_one_timer_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_one_timer = true;
            }
        }

        Ok(())
    }
}
