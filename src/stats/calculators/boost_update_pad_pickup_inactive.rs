use super::*;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub(super) fn update_inactive_boost_pad_pickup(
        &mut self,
        players: &PlayerFrameState,
        event: &BoostPadEvent,
        sample: &BoostUpdateSample,
    ) {
        let Some((player_id, player)) = boost_pickup_player(players, event) else {
            return;
        };
        let previous_boost_amount = self
            .previous_boost_amounts
            .get(player_id)
            .copied()
            .or(player.last_boost_amount)
            .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0));
        let respawn_amount = sample
            .respawn_amounts_by_player
            .get(player_id)
            .copied()
            .unwrap_or(0.0);
        let Some((collected_amount, pad_size)) = self.inactive_pickup_stats(
            player,
            &event.pad_id,
            previous_boost_amount,
            respawn_amount,
        ) else {
            return;
        };
        if !self
            .inactive_pickup_frames
            .insert((player_id.clone(), event.frame, pad_size))
        {
            return;
        }
        self.apply_inactive_pickup(
            BoostLedgerContext {
                frame: event.frame,
                time: event.time,
                boost_before: Some(previous_boost_amount),
                boost_after: player.boost_amount,
            },
            player_id,
            player.is_team_0,
            collected_amount,
            pad_size,
        );
        self.record_reported_pickup(PendingBoostPickupEvent {
            frame: event.frame,
            time: event.time,
            player_id: player_id.clone(),
            is_team_0: player.is_team_0,
            pad_type: pad_size.into(),
            field_half: Self::field_half_from_position(player.is_team_0, player.position()),
            activity: BoostPickupActivity::Inactive,
            boost_before: None,
            boost_after: None,
        });
    }
}

pub(super) fn boost_pickup_player<'a>(
    players: &'a PlayerFrameState,
    event: &'a BoostPadEvent,
) -> Option<(&'a PlayerId, &'a PlayerSample)> {
    let player_id = event.player.as_ref()?;
    let player = players
        .players
        .iter()
        .find(|player| &player.player_id == player_id)?;
    Some((player_id, player))
}
