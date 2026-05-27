use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_sample::BoostUpdateSample;

impl BoostCalculator {
    pub(super) fn update_player_boost_sample(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        context: &BoostUpdateContext,
        sample: &mut BoostUpdateSample,
    ) {
        let Some(boost_amount) = player.boost_amount else {
            return;
        };
        let previous_sample_boost_amount =
            self.previous_boost_amounts.get(&player.player_id).copied();
        let previous_boost_amount = current_previous_boost_amount(
            player,
            previous_sample_boost_amount,
            boost_amount,
            context,
        );
        let demo_respawn_ready = self.demo_respawn_ready(frame, player);
        if self.defer_pending_demo_respawn(player, previous_sample_boost_amount, demo_respawn_ready)
        {
            return;
        }

        self.record_inferred_pickups_for_player(
            frame,
            player,
            boost_amount,
            previous_sample_boost_amount,
            context,
            demo_respawn_ready,
        );
        self.record_boost_level_sample(frame, player, boost_amount, previous_boost_amount, context);
        let respawn_amount = self.apply_player_respawns(
            frame,
            player,
            boost_amount,
            previous_boost_amount,
            context,
            demo_respawn_ready,
        );
        sample
            .respawn_amounts_by_player
            .insert(player.player_id.clone(), respawn_amount);
        sample
            .current_boost_amounts
            .push((player.player_id.clone(), boost_amount));
    }

    fn demo_respawn_ready(&self, frame: &FrameInfo, player: &PlayerSample) -> bool {
        self.pending_demo_respawns
            .get(&player.player_id)
            .is_some_and(|pending| {
                player.rigid_body.is_some()
                    && frame.time - pending.demo_time >= DEMO_RESPAWN_WINDOW_SECONDS
            })
    }

    fn defer_pending_demo_respawn(
        &mut self,
        player: &PlayerSample,
        previous_sample_boost_amount: Option<f32>,
        demo_respawn_ready: bool,
    ) -> bool {
        if !self.pending_demo_respawns.contains_key(&player.player_id) || demo_respawn_ready {
            return false;
        }
        if let Some(pending) = self.pending_demo_respawns.get_mut(&player.player_id) {
            pending.pre_demo_boost_amount = pending
                .pre_demo_boost_amount
                .or(previous_sample_boost_amount);
        }
        true
    }
}

fn current_previous_boost_amount(
    player: &PlayerSample,
    previous_sample_boost_amount: Option<f32>,
    boost_amount: f32,
    context: &BoostUpdateContext,
) -> f32 {
    let previous_boost_amount = player
        .last_boost_amount
        .unwrap_or_else(|| previous_sample_boost_amount.unwrap_or(boost_amount));
    if context.boost_levels_resumed_this_sample {
        boost_amount
    } else {
        previous_boost_amount
    }
}
