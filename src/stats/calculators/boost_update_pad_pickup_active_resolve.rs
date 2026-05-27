use super::*;
use boost_update_context::BoostUpdateContext;

impl BoostCalculator {
    pub(super) fn pre_applied_pad_size(
        &self,
        event: &BoostPadEvent,
        player: &PlayerSample,
        collected_amount: f32,
    ) -> Option<BoostPadSize> {
        (collected_amount > 0.0)
            .then(|| {
                self.guess_pad_size_from_position(
                    &event.pad_id,
                    player.position().unwrap_or(glam::Vec3::ZERO),
                )
            })
            .flatten()
    }

    pub(super) fn resolve_and_record_active_pickup(
        &mut self,
        event: &BoostPadEvent,
        player_id: &PlayerId,
        player: &PlayerSample,
        pending_pickup: PendingBoostPickup,
        context: &BoostUpdateContext,
    ) {
        let Some(pad_size) = self.resolve_reported_pad_size(event, player, &pending_pickup) else {
            return;
        };
        let field_half = self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
        self.record_reported_pickup(PendingBoostPickupEvent {
            frame: event.frame,
            time: event.time,
            player_id: player_id.clone(),
            is_team_0: player.is_team_0,
            pad_type: pad_size.into(),
            field_half,
            activity: Self::activity_label(context.track_boost_pickups),
            boost_before: None,
            boost_after: None,
        });
    }

    fn resolve_reported_pad_size(
        &mut self,
        event: &BoostPadEvent,
        player: &PlayerSample,
        pending_pickup: &PendingBoostPickup,
    ) -> Option<BoostPadSize> {
        self.known_pad_sizes
            .get(&event.pad_id)
            .copied()
            .or_else(|| {
                let mut size = self.guess_pad_size_from_position(
                    &event.pad_id,
                    player.position().unwrap_or(glam::Vec3::ZERO),
                )?;
                if size == BoostPadSize::Small
                    && pending_pickup.pre_applied_collected_amount > SMALL_PAD_AMOUNT_RAW * 1.5
                {
                    size = BoostPadSize::Big;
                }
                self.known_pad_sizes.insert(event.pad_id.clone(), size);
                Some(size)
            })
    }
}
