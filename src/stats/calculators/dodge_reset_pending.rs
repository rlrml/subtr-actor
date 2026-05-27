use super::*;

impl DodgeResetCalculator {
    pub(super) fn prune_pending_resets(&mut self, players: &PlayerFrameState) {
        let grounded_players = self
            .pending_on_ball_resets
            .keys()
            .filter(|player_id| Self::player_is_grounded(players, player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in grounded_players {
            self.pending_on_ball_resets.remove(&player_id);
            self.pending_reset_dodge_started.remove(&player_id);
        }
    }

    pub(super) fn update_pending_reset_dodges(&mut self, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if player.dodge_active
                && !was_dodge_active
                && self.pending_on_ball_resets.contains_key(&player.player_id)
            {
                self.pending_reset_dodge_started
                    .insert(player.player_id.clone());
            }
        }
    }
}
