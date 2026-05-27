use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_pad_pickup_inactive::boost_pickup_player;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub(super) fn update_active_boost_pad_pickup(
        &mut self,
        players: &PlayerFrameState,
        event: &BoostPadEvent,
        sequence: u8,
        context: &BoostUpdateContext,
        sample: &BoostUpdateSample,
    ) {
        let Some((player_id, player)) = boost_pickup_player(players, event) else {
            return;
        };
        if self.should_skip_active_pickup(event, sequence, player_id, player) {
            return;
        }

        self.record_pad_observation(event, player);
        let pickup = self.build_pending_boost_pickup(event, player_id, player, sample);
        self.apply_pickup_collected_amount(
            BoostLedgerContext {
                frame: event.frame,
                time: event.time,
                boost_before: Some(pickup.previous_boost_amount),
                boost_after: player.boost_amount,
            },
            player_id,
            player.is_team_0,
            pickup.pre_applied_collected_amount,
            pickup.pre_applied_pad_size,
        );
        self.resolve_and_record_active_pickup(event, player_id, player, pickup, context);
    }

    fn should_skip_active_pickup(
        &mut self,
        event: &BoostPadEvent,
        sequence: u8,
        player_id: &PlayerId,
        player: &PlayerSample,
    ) -> bool {
        if self.unavailable_pad_is_recent(&event.pad_id, event.time, player.position()) {
            return true;
        }
        let pickup_key = (event.pad_id.clone(), player_id.clone());
        if self.pickup_frames.get(&pickup_key).copied() == Some(event.frame) {
            return true;
        }
        self.pickup_frames.insert(pickup_key, event.frame);
        if self.seen_pickup_sequence_is_recent(
            &event.pad_id,
            sequence,
            event.time,
            player.position(),
        ) {
            return true;
        }
        self.seen_pickup_sequence_times
            .insert((event.pad_id.clone(), sequence), event.time);
        false
    }
}
