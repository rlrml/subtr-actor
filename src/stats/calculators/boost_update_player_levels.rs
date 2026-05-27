use super::*;
use boost_update_context::BoostUpdateContext;
use boost_update_player_level_times::BoostLevelTimes;

impl BoostCalculator {
    pub(super) fn record_boost_level_sample(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        boost_amount: f32,
        previous_boost_amount: f32,
        context: &BoostUpdateContext,
    ) {
        if !context.track_boost_levels {
            return;
        }
        let boost_before =
            (!context.boost_levels_resumed_this_sample).then_some(previous_boost_amount);
        self.record_state_event(BoostStateEvent {
            frame: frame.frame_number,
            time: frame.time,
            player_id: player.player_id.clone(),
            is_team_0: player.is_team_0,
            boost_amount,
            boost_before,
        });

        let time = BoostLevelTimes::from_interval(frame.dt, previous_boost_amount, boost_amount);
        let stats = self
            .player_stats
            .entry(player.player_id.clone())
            .or_default();
        let team_stats = if player.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        time.apply(stats);
        time.apply(team_stats);
    }
}
