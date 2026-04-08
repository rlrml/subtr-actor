use super::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineFrameState {
    pub frame: Option<ReplayStatsFrame>,
}

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
        let fifty_fifty = ctx.get::<FiftyFiftyCalculator>()?;
        let possession = ctx.get::<PossessionCalculator>()?;
        let pressure = ctx.get::<PressureCalculator>()?;
        let rush = ctx.get::<RushCalculator>()?;
        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        let backboard = ctx.get::<BackboardCalculator>()?;
        let double_tap = ctx.get::<DoubleTapCalculator>()?;
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        let boost = ctx.get::<BoostCalculator>()?;
        let movement = ctx.get::<MovementCalculator>()?;
        let powerslide = ctx.get::<PowerslideCalculator>()?;
        let demo = ctx.get::<DemoCalculator>()?;
        Ok(TeamStatsSnapshot {
            fifty_fifty: fifty_fifty.stats().for_team(is_team_zero),
            possession: possession.stats().for_team(is_team_zero),
            pressure: pressure.stats().for_team(is_team_zero),
            rush: rush.stats().for_team(is_team_zero),
            core: if is_team_zero {
                match_stats.team_zero_stats()
            } else {
                match_stats.team_one_stats()
            },
            backboard: if is_team_zero {
                backboard.team_zero_stats().clone()
            } else {
                backboard.team_one_stats().clone()
            },
            double_tap: if is_team_zero {
                double_tap.team_zero_stats().clone()
            } else {
                double_tap.team_one_stats().clone()
            },
            ball_carry: if is_team_zero {
                ball_carry.team_zero_stats().clone()
            } else {
                ball_carry.team_one_stats().clone()
            },
            boost: if is_team_zero {
                boost.team_zero_stats().clone()
            } else {
                boost.team_one_stats().clone()
            },
            movement: if is_team_zero {
                movement.team_zero_stats().clone()
            } else {
                movement.team_one_stats().clone()
            },
            powerslide: if is_team_zero {
                powerslide.team_zero_stats().clone()
            } else {
                powerslide.team_one_stats().clone()
            },
            demo: if is_team_zero {
                demo.team_zero_stats().clone()
            } else {
                demo.team_one_stats().clone()
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
        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: Self::is_team_zero_player(replay_meta, player),
            core: ctx
                .get::<MatchStatsCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            backboard: ctx
                .get::<BackboardCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            ceiling_shot: ctx
                .get::<CeilingShotCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            double_tap: ctx
                .get::<DoubleTapCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            fifty_fifty: ctx
                .get::<FiftyFiftyCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            speed_flip: ctx
                .get::<SpeedFlipCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            touch: ctx
                .get::<TouchCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default()
                .with_complete_labeled_touch_counts(),
            musty_flick: ctx
                .get::<MustyFlickCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            dodge_reset: ctx
                .get::<DodgeResetCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            ball_carry: ctx
                .get::<BallCarryCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            boost: ctx
                .get::<BoostCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            movement: ctx
                .get::<MovementCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default()
                .with_complete_labeled_tracked_time(),
            positioning: ctx
                .get::<PositioningCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            powerslide: ctx
                .get::<PowerslideCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
            demo: ctx
                .get::<DemoCalculator>()?
                .player_stats()
                .get(player_id)
                .cloned()
                .unwrap_or_default(),
        })
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
            match_stats_dependency(),
            backboard_dependency(),
            ceiling_shot_dependency(),
            double_tap_dependency(),
            fifty_fifty_dependency(),
            possession_dependency(),
            pressure_dependency(),
            rush_dependency(),
            touch_dependency(),
            speed_flip_dependency(),
            musty_flick_dependency(),
            dodge_reset_dependency(),
            ball_carry_dependency(),
            boost_dependency(),
            movement_dependency(),
            positioning_dependency(),
            powerslide_dependency(),
            demo_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
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

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineFrameNode::new())
}
