use super::*;

impl StatsTimelineFrameNode {
    pub(super) fn is_team_zero_player(replay_meta: &ReplayMeta, player: &PlayerInfo) -> bool {
        replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    pub(super) fn update_snapshot(
        &mut self,
        ctx: &AnalysisStateContext<'_>,
    ) -> SubtrActorResult<()> {
        let replay_meta = self.replay_meta()?;
        let frame = ctx.get::<FrameInfo>()?;
        let gameplay = ctx.get::<GameplayState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.state.frame = Some(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            gameplay_phase: live_play_state.gameplay_phase,
            is_live_play: live_play_state.is_live_play,
            team_zero: self.team_snapshot(ctx, true)?,
            team_one: self.team_snapshot(ctx, false)?,
            players: replay_meta
                .player_order()
                .map(|player| self.player_snapshot(ctx, replay_meta, player))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        });
        Ok(())
    }
}
