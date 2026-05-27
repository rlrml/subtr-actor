use super::*;
use boost_update_context::BoostUpdateContext;

impl BoostCalculator {
    pub(super) fn apply_player_respawns(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        boost_amount: f32,
        previous_boost_amount: f32,
        context: &BoostUpdateContext,
        demo_respawn_supported: bool,
    ) -> f32 {
        let mut respawn_amount = 0.0;
        let first_seen_player = self
            .initial_respawn_awarded
            .insert(player.player_id.clone());
        if first_seen_player
            || (context.kickoff_phase_active
                && !self.kickoff_respawn_awarded.contains(&player.player_id))
        {
            respawn_amount += BOOST_KICKOFF_START_AMOUNT;
            self.kickoff_respawn_awarded
                .insert(player.player_id.clone());
        }

        if demo_respawn_supported {
            self.record_demo_reset_amount(player, previous_boost_amount);
            respawn_amount += BOOST_KICKOFF_START_AMOUNT;
            self.pending_demo_respawns.remove(&player.player_id);
        }

        if respawn_amount > 0.0 {
            self.apply_respawn_amount(
                BoostLedgerContext {
                    frame: frame.frame_number,
                    time: frame.time,
                    boost_before: Some(previous_boost_amount),
                    boost_after: Some(boost_amount),
                },
                &player.player_id,
                player.is_team_0,
                respawn_amount,
            );
        }
        respawn_amount
    }

    fn record_demo_reset_amount(&mut self, player: &PlayerSample, previous_boost_amount: f32) {
        if let Some(pending) = self.pending_demo_respawns.get(&player.player_id) {
            let demo_reset_amount = pending
                .pre_demo_boost_amount
                .unwrap_or(previous_boost_amount)
                .max(0.0);
            *self
                .demo_reset_boost_amounts
                .entry(player.player_id.clone())
                .or_default() += demo_reset_amount;
        }
    }
}
