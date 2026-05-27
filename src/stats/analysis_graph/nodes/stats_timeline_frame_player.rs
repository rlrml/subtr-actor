use super::*;
use std::collections::HashMap;

impl StatsTimelineFrameNode {
    pub(super) fn player_snapshot(
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
            core: player_stats(ctx, player_id, MatchStatsCalculator::player_stats)?,
            backboard: player_stats(ctx, player_id, BackboardCalculator::player_stats)?,
            ceiling_shot: player_stats(ctx, player_id, CeilingShotCalculator::player_stats)?,
            wall_aerial: player_stats(ctx, player_id, WallAerialCalculator::player_stats)?,
            wall_aerial_shot: player_stats(ctx, player_id, WallAerialShotCalculator::player_stats)?,
            double_tap: player_stats(ctx, player_id, DoubleTapCalculator::player_stats)?,
            one_timer: player_stats(ctx, player_id, OneTimerCalculator::player_stats)?,
            pass: player_stats(ctx, player_id, PassCalculator::player_stats)?,
            fifty_fifty: player_stats(ctx, player_id, FiftyFiftyCalculator::player_stats)?,
            speed_flip: player_stats(ctx, player_id, SpeedFlipCalculator::player_stats)?,
            half_flip: player_stats(ctx, player_id, HalfFlipCalculator::player_stats)?,
            wavedash: player_stats(ctx, player_id, WavedashCalculator::player_stats)?,
            touch: player_stats(ctx, player_id, TouchCalculator::player_stats)?,
            whiff: player_stats(ctx, player_id, WhiffCalculator::player_stats)?,
            flick: player_stats(ctx, player_id, FlickCalculator::player_stats)?,
            musty_flick: player_stats(ctx, player_id, MustyFlickCalculator::player_stats)?,
            dodge_reset: player_stats(ctx, player_id, DodgeResetCalculator::player_stats)?,
            ball_carry: player_stats(ctx, player_id, BallCarryCalculator::player_stats)?,
            air_dribble: player_stats(
                ctx,
                player_id,
                BallCarryCalculator::player_air_dribble_stats,
            )?,
            boost: player_stats(ctx, player_id, BoostCalculator::player_stats)?,
            bump: player_stats(ctx, player_id, BumpCalculator::player_stats)?,
            half_volley: player_stats(ctx, player_id, HalfVolleyCalculator::player_stats)?,
            movement: player_stats(ctx, player_id, MovementCalculator::player_stats)?,
            positioning: player_stats(ctx, player_id, PositioningCalculator::player_stats)?,
            rotation: player_stats(ctx, player_id, RotationCalculator::player_stats)?,
            powerslide: player_stats(ctx, player_id, PowerslideCalculator::player_stats)?,
            demo: player_stats(ctx, player_id, DemoCalculator::player_stats)?,
        })
    }
}

fn player_stats<C, T>(
    ctx: &AnalysisStateContext<'_>,
    player_id: &PlayerId,
    stats: impl Fn(&C) -> &HashMap<PlayerId, T>,
) -> SubtrActorResult<T>
where
    C: 'static,
    T: Clone + Default,
{
    Ok(stats(ctx.get::<C>()?)
        .get(player_id)
        .cloned()
        .unwrap_or_default())
}
