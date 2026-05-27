use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn replay_stats_frame(
        &self,
        frame: &StatsSnapshotFrame,
    ) -> SubtrActorResult<ReplayStatsFrame> {
        Ok(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: frame.game_state,
            ball_has_been_hit: frame.ball_has_been_hit,
            kickoff_countdown_time: frame.kickoff_countdown_time,
            gameplay_phase: frame.gameplay_phase,
            is_live_play: frame.is_live_play,
            team_zero: self.replay_team_stats(frame, "team_zero")?,
            team_one: self.replay_team_stats(frame, "team_one")?,
            players: self
                .replay_meta
                .player_order()
                .map(|player| self.replay_player_stats(frame, player))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        })
    }
}
