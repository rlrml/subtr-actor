use std::collections::{HashMap, HashSet};

use super::*;
use crate::stats::calculators::*;
use crate::{PlayerId, SubtrActorResult};

#[derive(Debug, Clone, Default)]
pub struct StatsProjectionState {
    pub core: CoreStatsAccumulator,
    pub backboard: BackboardStatsAccumulator,
    pub ceiling_shot: CeilingShotStatsAccumulator,
    pub wall_aerial: WallAerialStatsAccumulator,
    pub wall_aerial_shot: WallAerialShotStatsAccumulator,
    pub double_tap: DoubleTapStatsAccumulator,
    pub one_timer: OneTimerStatsAccumulator,
    pub pass: PassStatsAccumulator,
    pub fifty_fifty: FiftyFiftyStatsAccumulator,
    pub kickoff: KickoffStatsAccumulator,
    pub possession: PossessionStatsAccumulator,
    pub ball_half: BallHalfStatsAccumulator,
    pub territorial_pressure: TerritorialPressureStatsAccumulator,
    pub rotation: RotationStatsAccumulator,
    pub rush: RushStatsAccumulator,
    pub touch: TouchStatsAccumulator,
    pub whiff: WhiffStatsAccumulator,
    pub wavedash: WavedashStatsAccumulator,
    pub speed_flip: SpeedFlipStatsAccumulator,
    pub half_flip: HalfFlipStatsAccumulator,
    pub flick: FlickStatsAccumulator,
    pub musty_flick: MustyFlickStatsAccumulator,
    pub dodge_reset: DodgeResetStatsAccumulator,
    pub ball_carry: BallCarryStatsAccumulator,
    pub boost: BoostStatsAccumulator,
    pub bump: BumpStatsAccumulator,
    pub half_volley: HalfVolleyStatsAccumulator,
    pub movement: MovementStatsAccumulator,
    pub positioning: PositioningStatsAccumulator,
    pub powerslide: PowerslideStatsAccumulator,
    pub demo: DemoStatsAccumulator,
    pub center: CenterStatsAccumulator,
    pub controlled_play: ControlledPlayStatsAccumulator,
}

#[derive(Debug, Clone, Default)]
struct PowerslideProjectionState {
    active_players: HashMap<PlayerId, bool>,
    player_teams: HashMap<PlayerId, bool>,
}

impl PowerslideProjectionState {
    fn apply_frame(
        &mut self,
        stats: &mut PowerslideStatsAccumulator,
        frame: &FrameInfo,
        events: &[PowerslideEvent],
        counts_toward_motion: bool,
    ) {
        let mut started_this_frame = HashSet::new();
        for event in events {
            self.player_teams
                .insert(event.player.clone(), event.is_team_0);
            if event.active {
                stats.apply_sample(
                    &event.player,
                    event.is_team_0,
                    true,
                    false,
                    frame.dt,
                    counts_toward_motion,
                );
                self.active_players.insert(event.player.clone(), true);
                started_this_frame.insert(event.player.clone());
            } else {
                self.active_players.insert(event.player.clone(), false);
            }
        }

        let active_players = self
            .active_players
            .iter()
            .filter(|(player_id, active)| **active && !started_this_frame.contains(*player_id))
            .map(|(player_id, _)| player_id.clone())
            .collect::<Vec<_>>();
        for player_id in active_players {
            let Some(is_team_0) = self.player_teams.get(&player_id).copied() else {
                continue;
            };
            stats.apply_sample(
                &player_id,
                is_team_0,
                true,
                true,
                frame.dt,
                counts_toward_motion,
            );
        }
    }
}

/// Incrementally maintained movement-stats projection.
///
/// The movement calculator coalesces samples into a small set of in-progress
/// *pending* events (one per active player) that keep mutating until a player's
/// classification changes, at which point they finalize into the immutable,
/// append-only committed stream. The published per-frame snapshot has to
/// reflect committed + pending, but re-accumulating the entire committed
/// history every frame is O(n^2) over a replay and stalls on long,
/// movement-heavy replays.
///
/// Instead, each committed event is folded into a persistent `base` exactly
/// once (tracked by `committed_cursor`), and the bounded pending set is overlaid
/// on a clone of that base to produce the frame snapshot. `MovementStats`
/// accumulation is purely additive, so this is identical to a full rebuild while
/// keeping per-frame work proportional to (newly committed events + players).
#[derive(Debug, Clone, Default)]
struct IncrementalMovementProjection {
    base: MovementStatsAccumulator,
    committed_cursor: usize,
    /// Total committed events folded into `base` over this projection's
    /// lifetime. Tests assert this stays equal to the committed event count
    /// (each folded exactly once), which is what distinguishes the incremental
    /// fold from the previous quadratic per-frame rebuild.
    #[cfg(test)]
    committed_folds: usize,
}

impl IncrementalMovementProjection {
    /// Fold any newly committed events into `base`, then return the published
    /// snapshot as `base` plus the (bounded) pending overlay.
    fn project(
        &mut self,
        committed: &[MovementEvent],
        pending: &[MovementEvent],
    ) -> MovementStatsAccumulator {
        for event in committed.get(self.committed_cursor..).unwrap_or(&[]) {
            self.base.apply_event(event);
            #[cfg(test)]
            {
                self.committed_folds += 1;
            }
        }
        self.committed_cursor = committed.len();

        let mut snapshot = self.base.clone();
        for event in pending {
            snapshot.apply_event(event);
        }
        snapshot
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsProjectionNode {
    state: StatsProjectionState,
    cursors: StatsProjectionCursors,
    movement_projection: IncrementalMovementProjection,
    powerslide: PowerslideProjectionState,
    boost_current_amount_consistency: BoostCurrentAmountConsistencyTracker,
    last_powerslide_sample_frame: Option<usize>,
    territorial_pressure_tracked_time: f32,
    previous_live_play: Option<bool>,
}

#[derive(Debug, Clone, Default)]
struct StatsProjectionCursors {
    core_player: usize,
    core_player_goal_context: usize,
    backboard: usize,
    ceiling_shot: usize,
    wall_aerial: usize,
    wall_aerial_shot: usize,
    double_tap: usize,
    one_timer: usize,
    pass: usize,
    fifty_fifty: usize,
    kickoff: usize,
    possession: usize,
    ball_half: usize,
    rush: usize,
    touch: usize,
    whiff: usize,
    wavedash: usize,
    speed_flip: usize,
    half_flip: usize,
    flick: usize,
    musty_flick: usize,
    dodge_reset: usize,
    dodge_reset_flip_reset_outcome: usize,
    ball_carry: usize,
    bump: usize,
    half_volley: usize,
    powerslide: usize,
    demo_timeline: usize,
    center: usize,
    controlled_play: usize,
}

impl StatsProjectionNode {
    pub fn new() -> Self {
        Self::default()
    }

    fn begin_sample(&mut self, frame: &FrameInfo, live_play: bool) {
        self.state.backboard.begin_sample(frame);
        self.state.center.begin_sample(frame);
        self.state.double_tap.begin_sample(frame);
        self.state.half_volley.begin_sample(frame);
        self.state.one_timer.begin_sample(frame);
        self.state.pass.begin_sample(frame);
        self.state.wall_aerial.begin_sample(frame);
        self.state.wall_aerial_shot.begin_sample(frame);

        if !live_play {
            self.state.center.clear_current_last();
            self.state.one_timer.clear_current_last();
            self.state.pass.clear_current_last();
            self.state.half_volley.reset_current_last_event_marker();
            self.state.touch.set_current_last_touch_player(None);
            self.state.wall_aerial.reset_current_last_event_marker();
            self.state
                .wall_aerial_shot
                .reset_current_last_event_marker();
        }

        if live_play && self.previous_live_play == Some(false) {
            self.state.ceiling_shot.reset_current_last_event_marker();
            self.state.flick.reset_current_last_event_marker();
            self.state.half_flip.reset_current_last_event_marker();
            self.state.musty_flick.reset_current_last_event_marker();
            self.state.wavedash.reset_current_last_event_marker();
            self.state.whiff.reset_current_last_event_marker();
        }

        if live_play {
            self.state.ceiling_shot.begin_sample(frame);
            self.state.flick.begin_sample(frame);
            self.state.half_flip.begin_sample(frame);
            self.state.musty_flick.begin_sample(frame);
            self.state.touch.begin_sample(frame);
            self.state.wavedash.begin_sample(frame);
            self.state.whiff.begin_sample(frame);
        }
    }

    fn finish_sample(&mut self) {
        self.state.center.finish_sample();
        self.state.double_tap.finish_sample();
        self.state.one_timer.finish_sample();
        self.state.pass.finish_sample();
        self.state.half_volley.restore_current_last_event_marker();
        self.state.touch.restore_current_last_touch_marker();
        self.state.wall_aerial.restore_current_last_event_marker();
        self.state
            .wall_aerial_shot
            .restore_current_last_event_marker();
        self.state.whiff.restore_current_last_event_marker();
    }

    fn events_since<'a, E>(cursor: &mut usize, events: &'a [E]) -> &'a [E] {
        let start = (*cursor).min(events.len());
        *cursor = events.len();
        &events[start..]
    }

    fn check_boost_current_amount_consistency(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            if player.boost_active {
                continue;
            }
            let Some(observed_byte) = player
                .last_boost_amount
                .map(|amount| amount.round().clamp(0.0, BOOST_MAX_AMOUNT) as u8)
            else {
                continue;
            };
            let stats = self.state.boost.player_stats_for(&player.player_id);
            self.boost_current_amount_consistency.observe(
                frame.frame_number,
                frame.time,
                &player.player_id,
                &stats,
                observed_byte,
            );
        }
    }

    fn warn_for_unresolved_boost_current_amount_drift(&self) {
        for warning in self.boost_current_amount_consistency.unresolved_warnings() {
            log::warn!(
                "Boost invariant violation for player {:?} at frame {} (t={:.3}): {}",
                warning.player_id,
                warning.frame,
                warning.time,
                warning.message(),
            );
        }
    }

    fn project_frame(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        let live_play = live_play_state.is_live_play;
        let counts_toward_powerslide_motion = matches!(
            live_play_state.gameplay_phase,
            GameplayPhase::ActivePlay | GameplayPhase::KickoffWaitingForTouch
        );
        let gameplay = ctx.get::<GameplayState>()?;
        let speed_flip_stats_advance = live_play || gameplay.ball_has_been_hit == Some(false);
        let should_sample_powerslide =
            self.last_powerslide_sample_frame != Some(frame.frame_number);
        self.begin_sample(frame, live_play);

        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        for event in Self::events_since(
            &mut self.cursors.core_player,
            match_stats.core_player_events(),
        ) {
            self.state.core.apply_scoreboard_event(event);
        }
        for event in Self::events_since(
            &mut self.cursors.core_player_goal_context,
            match_stats.core_player_goal_context_events(),
        ) {
            self.state.core.apply_goal_context_event(event);
        }

        let backboard = ctx.get::<BackboardCalculator>()?;
        self.state.backboard.apply_events(
            frame,
            Self::events_since(&mut self.cursors.backboard, backboard.events()),
        );

        let ceiling_shot = ctx.get::<CeilingShotCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.ceiling_shot, ceiling_shot.events()) {
                self.state.ceiling_shot.apply_event(event, frame);
            }
        }
        let wall_aerial = ctx.get::<WallAerialCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.wall_aerial, wall_aerial.events()) {
                self.state.wall_aerial.apply_event(event, frame);
            }
        }
        let wall_aerial_shot = ctx.get::<WallAerialShotCalculator>()?;
        if live_play {
            for event in Self::events_since(
                &mut self.cursors.wall_aerial_shot,
                wall_aerial_shot.events(),
            ) {
                self.state.wall_aerial_shot.apply_event(event, frame);
            }
        }
        let double_tap = ctx.get::<DoubleTapCalculator>()?;
        for event in Self::events_since(&mut self.cursors.double_tap, double_tap.events()) {
            self.state.double_tap.apply_event(frame, event);
        }
        let one_timer = ctx.get::<OneTimerCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.one_timer, one_timer.events()) {
                self.state.one_timer.apply_event(frame, event);
            }
        }
        let pass = ctx.get::<PassCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.pass, pass.events()) {
                self.state.pass.apply_event(frame, event);
            }
        }
        let fifty_fifty = ctx.get::<FiftyFiftyCalculator>()?;
        for event in Self::events_since(&mut self.cursors.fifty_fifty, fifty_fifty.events()) {
            self.state.fifty_fifty.apply_event(event);
        }
        let kickoff = ctx.get::<KickoffCalculator>()?;
        for event in Self::events_since(&mut self.cursors.kickoff, kickoff.events()) {
            self.state.kickoff.apply_event(event);
        }
        let possession = ctx.get::<PossessionCalculator>()?;
        let projected_possession_events = possession.projected_events();
        self.state.possession = PossessionStatsAccumulator::default();
        for event in projected_possession_events.iter() {
            self.state.possession.apply_event(event);
        }
        self.cursors.possession = possession.events().len();
        let ball_half = ctx.get::<BallHalfCalculator>()?;
        let projected_ball_half_events = ball_half.projected_events();
        self.state.ball_half = BallHalfStatsAccumulator::default();
        for event in projected_ball_half_events.iter() {
            self.state.ball_half.apply_event(event);
        }
        self.cursors.ball_half = ball_half.events().len();
        let territorial_pressure = ctx.get::<TerritorialPressureCalculator>()?;
        if live_play {
            self.territorial_pressure_tracked_time += frame.dt;
        }
        let projected_territorial_pressure_events = territorial_pressure.projected_events();
        self.state.territorial_pressure = TerritorialPressureStatsAccumulator::default();
        self.state
            .territorial_pressure
            .set_tracked_time(self.territorial_pressure_tracked_time);
        for event in projected_territorial_pressure_events.iter() {
            self.state.territorial_pressure.apply_event(event);
        }
        let rotation = ctx.get::<RotationCalculator>()?;
        self.state.rotation = RotationStatsAccumulator::with_first_man_stint_end_grace_seconds(
            rotation.config().first_man_stint_end_grace_seconds,
        );
        for event in rotation.role_events().iter() {
            self.state.rotation.apply_role_event(event);
        }
        for event in rotation.first_man_change_events() {
            self.state.rotation.apply_first_man_change_event(event);
        }
        let rush = ctx.get::<RushCalculator>()?;
        for event in Self::events_since(&mut self.cursors.rush, rush.events()) {
            self.state.rush.apply_event(event);
        }
        let touch = ctx.get::<TouchCalculator>()?;
        if live_play || self.cursors.touch != touch.events().len() {
            self.state.touch = TouchStatsAccumulator::default();
            for event in touch.events() {
                self.state.touch.apply_touch_event(event, frame);
            }
            self.cursors.touch = touch.events().len();
        }
        let whiff = ctx.get::<WhiffCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.whiff, whiff.events()) {
                self.state.whiff.apply_event(event, frame);
            }
        }
        let wavedash = ctx.get::<WavedashCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.wavedash, wavedash.events()) {
                self.state.wavedash.apply_event(event);
            }
        }
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        if speed_flip_stats_advance {
            self.state.speed_flip.begin_sample(frame);
            for event in Self::events_since(&mut self.cursors.speed_flip, speed_flip.events()) {
                self.state.speed_flip.apply_event(event);
            }
        }
        let half_flip = ctx.get::<HalfFlipCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.half_flip, half_flip.events()) {
                self.state.half_flip.apply_event(event);
            }
        }
        let flick = ctx.get::<FlickCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.flick, flick.events()) {
                self.state.flick.apply_event(event, frame);
            }
        }
        let musty_flick = ctx.get::<MustyFlickCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.musty_flick, musty_flick.events()) {
                self.state.musty_flick.apply_event(event, frame);
            }
        }
        let dodge_reset = ctx.get::<DodgeResetCalculator>()?;
        for event in Self::events_since(&mut self.cursors.dodge_reset, dodge_reset.events()) {
            self.state.dodge_reset.apply_event(event);
        }
        for event in Self::events_since(
            &mut self.cursors.dodge_reset_flip_reset_outcome,
            dodge_reset.flip_reset_outcome_events(),
        ) {
            self.state.dodge_reset.apply_flip_reset_outcome_event(event);
        }
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        for event in Self::events_since(&mut self.cursors.ball_carry, ball_carry.carry_events()) {
            self.state.ball_carry.apply_event(event);
        }
        let boost = ctx.get::<BoostCalculator>()?;
        // The boost calculator now accumulates BoostStats directly as it processes frames, so we
        // mirror its accumulator instead of replaying projected ledger/state events.
        self.state.boost = boost.boost_stats().clone();
        if live_play {
            self.check_boost_current_amount_consistency(frame, players);
        }
        let bump = ctx.get::<BumpCalculator>()?;
        for event in Self::events_since(&mut self.cursors.bump, bump.events()) {
            self.state.bump.apply_event(event);
        }
        let half_volley = ctx.get::<HalfVolleyCalculator>()?;
        if live_play {
            for event in Self::events_since(&mut self.cursors.half_volley, half_volley.events()) {
                self.state.half_volley.apply_event(event, frame);
            }
        }
        let movement = ctx.get::<MovementCalculator>()?;
        self.state.movement = self
            .movement_projection
            .project(movement.events(), &movement.pending_events());
        let positioning = ctx.get::<PositioningCalculator>()?;
        self.state.positioning = PositioningStatsAccumulator::default();
        for event in positioning.activity_events().iter() {
            self.state.positioning.apply_activity_event(event);
        }
        for event in positioning.field_third_events().iter() {
            self.state.positioning.apply_field_third_event(event);
        }
        for event in positioning.field_half_events().iter() {
            self.state.positioning.apply_field_half_event(event);
        }
        for event in positioning.ball_depth_events().iter() {
            self.state.positioning.apply_ball_depth_event(event);
        }
        for event in positioning.depth_role_events().iter() {
            self.state.positioning.apply_depth_role_event(event);
        }
        for event in positioning.ball_proximity_events().iter() {
            self.state.positioning.apply_ball_proximity_event(event);
        }
        for (player, signal) in positioning.signals() {
            self.state.positioning.apply_signal(player, signal);
        }
        let powerslide = ctx.get::<PowerslideCalculator>()?;
        let powerslide_events =
            Self::events_since(&mut self.cursors.powerslide, powerslide.events());
        if should_sample_powerslide {
            self.powerslide.apply_frame(
                &mut self.state.powerslide,
                frame,
                powerslide_events,
                counts_toward_powerslide_motion,
            );
            self.last_powerslide_sample_frame = Some(frame.frame_number);
        }
        let demo = ctx.get::<DemoCalculator>()?;
        for event in Self::events_since(&mut self.cursors.demo_timeline, demo.timeline()) {
            self.state.demo.apply_timeline_event(event);
        }
        let center = ctx.get::<CenterCalculator>()?;
        for event in Self::events_since(&mut self.cursors.center, center.events()) {
            self.state.center.apply_event(frame, event);
        }
        let controlled_play = ctx.get::<ControlledPlayCalculator>()?;
        for event in Self::events_since(&mut self.cursors.controlled_play, controlled_play.events())
        {
            self.state.controlled_play.apply_event(event);
        }

        self.finish_sample();
        self.previous_live_play = Some(live_play);
        Ok(())
    }
}

impl AnalysisNode for StatsProjectionNode {
    type State = StatsProjectionState;

    fn name(&self) -> &'static str {
        "stats_projection"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            live_play_dependency(),
            player_frame_state_dependency(),
            match_stats_dependency(),
            backboard_dependency(),
            ceiling_shot_dependency(),
            wall_aerial_dependency(),
            wall_aerial_shot_dependency(),
            double_tap_dependency(),
            one_timer_dependency(),
            pass_dependency(),
            fifty_fifty_dependency(),
            kickoff_dependency(),
            possession_dependency(),
            ball_half_dependency(),
            territorial_pressure_dependency(),
            rotation_dependency(),
            rush_dependency(),
            touch_dependency(),
            whiff_dependency(),
            wavedash_dependency(),
            speed_flip_dependency(),
            half_flip_dependency(),
            flick_dependency(),
            musty_flick_dependency(),
            dodge_reset_dependency(),
            ball_carry_dependency(),
            boost_dependency(),
            bump_dependency(),
            half_volley_dependency(),
            movement_dependency(),
            positioning_dependency(),
            powerslide_dependency(),
            demo_dependency(),
            center_dependency(),
            controlled_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.project_frame(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.project_frame(ctx)?;
        self.warn_for_unresolved_boost_current_amount_drift();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsProjectionNode::new())
}

#[cfg(test)]
#[path = "stats_projection_tests.rs"]
mod tests;
