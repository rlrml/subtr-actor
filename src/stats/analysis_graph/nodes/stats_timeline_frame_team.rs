use super::stats_timeline_frame_team_helpers::{team_calc, team_value};
use super::*;

macro_rules! team_calc {
    ($ctx:expr, $is_team_zero:expr, $calculator:ty) => {
        team_calc(
            $ctx,
            $is_team_zero,
            <$calculator>::team_zero_stats,
            <$calculator>::team_one_stats,
        )?
    };
}

impl StatsTimelineFrameNode {
    pub(super) fn team_snapshot(
        &self,
        ctx: &AnalysisStateContext<'_>,
        is_team_zero: bool,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        Ok(TeamStatsSnapshot {
            fifty_fifty: ctx
                .get::<FiftyFiftyCalculator>()?
                .stats()
                .for_team(is_team_zero),
            possession: ctx
                .get::<PossessionCalculator>()?
                .stats()
                .for_team(is_team_zero),
            pressure: ctx
                .get::<PressureCalculator>()?
                .stats()
                .for_team(is_team_zero),
            territorial_pressure: ctx
                .get::<TerritorialPressureCalculator>()?
                .stats()
                .for_team(is_team_zero),
            rotation: team_calc!(ctx, is_team_zero, RotationCalculator),
            rush: ctx.get::<RushCalculator>()?.stats().for_team(is_team_zero),
            core: if is_team_zero {
                match_stats.team_zero_stats()
            } else {
                match_stats.team_one_stats()
            },
            backboard: team_calc!(ctx, is_team_zero, BackboardCalculator),
            double_tap: team_calc!(ctx, is_team_zero, DoubleTapCalculator),
            one_timer: team_calc!(ctx, is_team_zero, OneTimerCalculator),
            pass: team_calc!(ctx, is_team_zero, PassCalculator),
            ball_carry: team_value(
                ball_carry.team_zero_stats(),
                ball_carry.team_one_stats(),
                is_team_zero,
            ),
            air_dribble: team_value(
                ball_carry.team_zero_air_dribble_stats(),
                ball_carry.team_one_air_dribble_stats(),
                is_team_zero,
            ),
            boost: team_calc!(ctx, is_team_zero, BoostCalculator),
            bump: team_calc!(ctx, is_team_zero, BumpCalculator),
            half_volley: team_calc!(ctx, is_team_zero, HalfVolleyCalculator),
            movement: team_calc!(ctx, is_team_zero, MovementCalculator),
            powerslide: team_calc!(ctx, is_team_zero, PowerslideCalculator),
            demo: team_calc!(ctx, is_team_zero, DemoCalculator),
        })
    }
}
