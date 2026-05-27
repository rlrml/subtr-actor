use super::*;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub(super) fn record_pad_observation(&mut self, event: &BoostPadEvent, player: &PlayerSample) {
        self.unavailable_pads.insert(event.pad_id.clone());
        self.last_pickup_times
            .insert(event.pad_id.clone(), event.time);
        if let Some(position) = player.position() {
            self.observed_pad_positions
                .entry(event.pad_id.clone())
                .or_default()
                .observe(position);
        }
    }

    pub(super) fn build_pending_boost_pickup(
        &self,
        event: &BoostPadEvent,
        player_id: &PlayerId,
        player: &PlayerSample,
        sample: &BoostUpdateSample,
    ) -> PendingBoostPickup {
        let previous_boost_amount = player.last_boost_amount.unwrap_or_else(|| {
            self.previous_boost_amounts
                .get(player_id)
                .copied()
                .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0))
        });
        let pre_applied_collected_amount =
            self.pre_applied_collected_amount(player_id, player, previous_boost_amount, sample);
        PendingBoostPickup {
            frame: event.frame,
            time: event.time,
            player_id: player_id.clone(),
            is_team_0: player.is_team_0,
            previous_boost_amount,
            pre_applied_collected_amount,
            pre_applied_pad_size: self.pre_applied_pad_size(
                event,
                player,
                pre_applied_collected_amount,
            ),
            player_position: player.position().unwrap_or(glam::Vec3::ZERO),
            boost_before: Some(previous_boost_amount),
            boost_after: player.boost_amount,
        }
    }

    fn pre_applied_collected_amount(
        &self,
        player_id: &PlayerId,
        player: &PlayerSample,
        previous_boost_amount: f32,
        sample: &BoostUpdateSample,
    ) -> f32 {
        if sample.pickup_counts_by_player.get(player_id).copied() != Some(1) {
            return 0.0;
        }
        self.previous_boost_amounts
            .get(player_id)
            .copied()
            .map(|previous_sample_boost_amount| {
                let respawn_amount = sample
                    .respawn_amounts_by_player
                    .get(player_id)
                    .copied()
                    .unwrap_or(0.0);
                (player.boost_amount.unwrap_or(previous_boost_amount)
                    - previous_sample_boost_amount
                    - respawn_amount)
                    .max(0.0)
            })
            .unwrap_or(0.0)
    }
}
