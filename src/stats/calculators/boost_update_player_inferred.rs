use super::*;
use boost_update_context::BoostUpdateContext;

impl BoostCalculator {
    pub(super) fn record_inferred_pickups_for_player(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        boost_amount: f32,
        previous_sample_boost_amount: Option<f32>,
        context: &BoostUpdateContext,
        demo_respawn_supported: bool,
    ) {
        let Some(previous_sample_boost_amount) = previous_sample_boost_amount else {
            return;
        };
        let reasons = Self::classify_boost_increase_reasons(
            previous_sample_boost_amount,
            boost_amount,
            context.kickoff_phase_active,
            demo_respawn_supported,
        );
        for reason in reasons {
            if let Ok(pad_type) = BoostPickupPadType::try_from(reason) {
                self.record_inferred_pickup(PendingBoostPickupEvent {
                    frame: frame.frame_number,
                    time: frame.time,
                    player_id: player.player_id.clone(),
                    is_team_0: player.is_team_0,
                    pad_type,
                    field_half: Self::field_half_from_position(player.is_team_0, player.position()),
                    activity: Self::activity_label(context.live_play),
                    boost_before: Some(previous_sample_boost_amount),
                    boost_after: Some(boost_amount),
                });
            }
        }
    }
}
