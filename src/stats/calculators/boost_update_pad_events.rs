use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub(super) fn inactive_pickup_stats(
        &self,
        player: &PlayerSample,
        pad_id: &str,
        previous_boost_amount: f32,
        respawn_amount: f32,
    ) -> Option<(f32, BoostPadSize)> {
        let pad_size = self
            .known_pad_sizes
            .get(pad_id)
            .copied()
            .or_else(|| self.guess_pad_size_from_position(pad_id, player.position()?))?;
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let capacity_limited_gain = (BOOST_MAX_AMOUNT - previous_boost_amount)
            .min(nominal_gain)
            .max(0.0);
        let observed_gain = player
            .boost_amount
            .map(|amount| (amount - previous_boost_amount - respawn_amount).max(0.0))
            .unwrap_or(0.0);
        (observed_gain > 1.0).then_some((
            capacity_limited_gain.max(observed_gain).min(nominal_gain),
            pad_size,
        ))
    }

    pub(super) fn update_boost_pad_events(
        &mut self,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        context: &BoostUpdateContext,
        sample: &BoostUpdateSample,
    ) {
        for event in &events.boost_pad_events {
            match event.kind {
                BoostPadEventKind::PickedUp { sequence } => {
                    self.update_boost_pad_pickup_event(players, event, sequence, context, sample);
                }
                BoostPadEventKind::Available => {
                    self.update_boost_pad_available_event(event);
                }
            }
        }
    }

    fn update_boost_pad_available_event(&mut self, event: &BoostPadEvent) {
        if let Some(pad_size) = self.known_pad_sizes.get(&event.pad_id).copied() {
            let Some(last_pickup_time) = self.last_pickup_times.get(&event.pad_id) else {
                return;
            };
            if event.time - *last_pickup_time < Self::pad_respawn_time_seconds(pad_size) {
                return;
            }
        }
        self.unavailable_pads.remove(&event.pad_id);
    }

    fn update_boost_pad_pickup_event(
        &mut self,
        players: &PlayerFrameState,
        event: &BoostPadEvent,
        sequence: u8,
        context: &BoostUpdateContext,
        sample: &BoostUpdateSample,
    ) {
        if !context.track_boost_pickups && !self.config.include_non_live_pickups {
            self.update_inactive_boost_pad_pickup(players, event, sample);
        } else {
            self.update_active_boost_pad_pickup(players, event, sequence, context, sample);
        }
    }
}
