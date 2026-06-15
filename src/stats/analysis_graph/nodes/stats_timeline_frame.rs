use super::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineFrameState {
    pub frame: Option<ReplayStatsFrame>,
}

/// Terminal materialization node for the full stats timeline frame export.
///
/// This node aggregates many concrete calculator states into the typed
/// `ReplayStatsFrame` DTO for serialization and UI/client compatibility. It is
/// not a shared data provider for other analysis nodes; cross-node data flow
/// should stay on explicit dependencies on the specific upstream calculator or
/// state node.
pub struct StatsTimelineFrameNode {
    replay_meta: Option<ReplayMeta>,
    state: StatsTimelineFrameState,
}

impl StatsTimelineFrameNode {
    pub fn new() -> Self {
        Self {
            replay_meta: None,
            state: StatsTimelineFrameState::default(),
        }
    }

    fn replay_meta(&self) -> SubtrActorResult<&ReplayMeta> {
        self.replay_meta.as_ref().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing ReplayMeta state while building timeline frame".to_owned(),
            ))
        })
    }

    fn is_team_zero_player(replay_meta: &ReplayMeta, player: &PlayerInfo) -> bool {
        replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    fn team_snapshot(
        &self,
        ctx: &AnalysisStateContext<'_>,
        is_team_zero: bool,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        let projection = ctx.get::<StatsProjectionState>()?;
        Ok(TeamStatsSnapshot {
            fifty_fifty: projection.fifty_fifty.stats().for_team(is_team_zero),
            kickoff: projection.kickoff.stats().for_team(is_team_zero),
            possession: projection.possession.stats().for_team(is_team_zero),
            ball_half: projection.ball_half.stats().for_team(is_team_zero),
            territorial_pressure: projection
                .territorial_pressure
                .stats()
                .for_team(is_team_zero),
            rotation: if is_team_zero {
                projection.rotation.team_zero_stats().clone()
            } else {
                projection.rotation.team_one_stats().clone()
            },
            rush: projection.rush.stats().for_team(is_team_zero),
            core: if is_team_zero {
                projection.core.team_zero_stats()
            } else {
                projection.core.team_one_stats()
            },
            backboard: if is_team_zero {
                projection.backboard.team_zero_stats().clone()
            } else {
                projection.backboard.team_one_stats().clone()
            },
            double_tap: if is_team_zero {
                projection.double_tap.team_zero_stats().clone()
            } else {
                projection.double_tap.team_one_stats().clone()
            },
            one_timer: if is_team_zero {
                projection.one_timer.team_zero_stats().clone()
            } else {
                projection.one_timer.team_one_stats().clone()
            },
            pass: if is_team_zero {
                projection.pass.team_zero_stats().clone()
            } else {
                projection.pass.team_one_stats().clone()
            },
            ball_carry: if is_team_zero {
                projection.ball_carry.team_zero_stats().clone()
            } else {
                projection.ball_carry.team_one_stats().clone()
            },
            controlled_play: if is_team_zero {
                projection.controlled_play.team_zero_stats().clone()
            } else {
                projection.controlled_play.team_one_stats().clone()
            },
            air_dribble: if is_team_zero {
                projection.ball_carry.team_zero_air_dribble_stats().clone()
            } else {
                projection.ball_carry.team_one_air_dribble_stats().clone()
            },
            boost: if is_team_zero {
                projection.boost.team_zero_stats().clone()
            } else {
                projection.boost.team_one_stats().clone()
            },
            bump: if is_team_zero {
                projection.bump.team_zero_stats().clone()
            } else {
                projection.bump.team_one_stats().clone()
            },
            half_volley: if is_team_zero {
                projection.half_volley.team_zero_stats().clone()
            } else {
                projection.half_volley.team_one_stats().clone()
            },
            movement: if is_team_zero {
                projection.movement.team_zero_stats().clone()
            } else {
                projection.movement.team_one_stats().clone()
            },
            positioning: if is_team_zero {
                projection.positioning.team_zero_stats().clone()
            } else {
                projection.positioning.team_one_stats().clone()
            },
            powerslide: if is_team_zero {
                projection.powerslide.team_zero_stats().clone()
            } else {
                projection.powerslide.team_one_stats().clone()
            },
            demo: if is_team_zero {
                projection.demo.team_zero_stats().clone()
            } else {
                projection.demo.team_one_stats().clone()
            },
        })
    }

    fn player_snapshot(
        &self,
        ctx: &AnalysisStateContext<'_>,
        replay_meta: &ReplayMeta,
        player: &PlayerInfo,
    ) -> SubtrActorResult<PlayerStatsSnapshot> {
        let player_id = &player.remote_id;
        let projection = ctx.get::<StatsProjectionState>()?;
        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: Self::is_team_zero_player(replay_meta, player),
            core: projection
                .core
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            backboard: projection
                .backboard
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            ceiling_shot: projection
                .ceiling_shot
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            wall_aerial: projection
                .wall_aerial
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            wall_aerial_shot: projection
                .wall_aerial_shot
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            double_tap: projection
                .double_tap
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            one_timer: projection
                .one_timer
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            pass: projection
                .pass
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            fifty_fifty: projection
                .fifty_fifty
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            kickoff: projection
                .kickoff
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            speed_flip: projection
                .speed_flip
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            half_flip: projection
                .half_flip
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            wavedash: projection
                .wavedash
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            touch: projection
                .touch
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            whiff: projection
                .whiff
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            flick: projection
                .flick
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            musty_flick: projection
                .musty_flick
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            dodge_reset: projection
                .dodge_reset
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            ball_carry: projection
                .ball_carry
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            controlled_play: projection
                .controlled_play
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            air_dribble: projection
                .ball_carry
                .player_air_dribble_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            boost: projection
                .boost
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            bump: projection
                .bump
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            half_volley: projection
                .half_volley
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            movement: projection
                .movement
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            positioning: projection
                .positioning
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            rotation: projection
                .rotation
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            powerslide: projection
                .powerslide
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            demo: projection
                .demo
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
        })
    }

    fn update_snapshot(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
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

impl Default for StatsTimelineFrameNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for StatsTimelineFrameNode {
    type State = StatsTimelineFrameState;

    fn name(&self) -> &'static str {
        "stats_timeline_frame"
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.replay_meta = Some(meta.clone());
        Ok(())
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            live_play_dependency(),
            stats_projection_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_snapshot(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_snapshot(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}
