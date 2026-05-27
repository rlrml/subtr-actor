use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.pending_inferred_pickups.clear();
        Ok(())
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        let context = BoostUpdateContext::new(self, gameplay, live_play);
        self.begin_boost_update_sample(&context, events);

        let mut sample = BoostUpdateSample::from_events(events);
        for player in &players.players {
            self.update_player_boost_sample(frame, player, &context, &mut sample);
        }

        self.update_boost_pad_events(players, events, &context, &sample);
        self.flush_stale_pickup_comparisons(frame.frame_number);
        self.update_used_boost(frame, players, vertical_state, &context);
        self.finish_boost_update_sample(frame, players, &context, sample);
        Ok(())
    }
}
