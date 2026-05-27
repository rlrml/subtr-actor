use super::collector_event::StatsTimelineEventCollector;
use crate::*;
use std::collections::BTreeMap;

impl StatsTimelineEventCollector {
    fn replay_meta(&self) -> SubtrActorResult<&ReplayMeta> {
        self.replay_meta
            .as_ref()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))
    }

    fn is_team_zero_player(replay_meta: &ReplayMeta, player: &PlayerInfo) -> bool {
        replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    pub(super) fn snapshot_frame_scaffold(&self) -> SubtrActorResult<ReplayStatsFrameScaffold> {
        let replay_meta = self.replay_meta()?;
        let frame = self.graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while building stats timeline frame scaffold".to_owned(),
            ))
        })?;
        let gameplay = self.graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while building stats timeline frame scaffold"
                    .to_owned(),
            ))
        })?;
        let live_play_state = self.graph.state::<LivePlayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing LivePlayState state while building stats timeline frame scaffold"
                    .to_owned(),
            ))
        })?;

        Ok(ReplayStatsFrameScaffold {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            gameplay_phase: live_play_state.gameplay_phase,
            is_live_play: live_play_state.is_live_play,
            team_zero: BTreeMap::new(),
            team_one: BTreeMap::new(),
            players: replay_meta
                .player_order()
                .map(|player| ReplayStatsPlayerIdentity {
                    player_id: player.remote_id.clone(),
                    name: player.name.clone(),
                    is_team_0: Self::is_team_zero_player(replay_meta, player),
                })
                .collect(),
        })
    }
}
