use std::collections::{HashMap, HashSet};

use super::*;
use crate::stats::calculators::*;
use crate::{PlayerId, SubtrActorResult};

#[derive(Debug, Clone, Default)]
pub struct StatsProjectionState {
    pub core: CoreStatsAccumulator,
    pub core_team_events: Vec<CoreTeamStatsEvent>,
    pub backboard: BackboardStatsAccumulator,
    pub ceiling_shot: CeilingShotStatsAccumulator,
    pub wall_aerial: WallAerialStatsAccumulator,
    pub wall_aerial_shot: WallAerialShotStatsAccumulator,
    pub double_tap: DoubleTapStatsAccumulator,
    pub one_timer: OneTimerStatsAccumulator,
    pub pass: PassStatsAccumulator,
    pub fifty_fifty: FiftyFiftyStatsAccumulator,
    pub possession: PossessionStatsAccumulator,
    pub pressure: PressureStatsAccumulator,
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
        live_play: bool,
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
                    live_play,
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
            stats.apply_sample(&player_id, is_team_0, true, true, frame.dt, live_play);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsProjectionNode {
    state: StatsProjectionState,
    cursors: StatsProjectionCursors,
    powerslide: PowerslideProjectionState,
    last_powerslide_sample_frame: Option<usize>,
}

#[derive(Debug, Clone, Default)]
struct StatsProjectionCursors {
    core_player: usize,
    backboard: usize,
    ceiling_shot: usize,
    wall_aerial: usize,
    wall_aerial_shot: usize,
    double_tap: usize,
    one_timer: usize,
    pass: usize,
    fifty_fifty: usize,
    possession: usize,
    pressure: usize,
    territorial_pressure_stats: usize,
    rotation_player: usize,
    rotation_team: usize,
    rush: usize,
    touch: usize,
    touch_ball_movement: usize,
    whiff: usize,
    wavedash: usize,
    speed_flip: usize,
    half_flip: usize,
    flick: usize,
    musty_flick: usize,
    dodge_reset: usize,
    ball_carry: usize,
    boost_stats: usize,
    bump: usize,
    half_volley: usize,
    movement: usize,
    positioning: usize,
    powerslide: usize,
    demo_timeline: usize,
    center: usize,
}

impl StatsProjectionNode {
    pub fn new() -> Self {
        Self::default()
    }

    fn begin_sample(&mut self, frame: &FrameInfo, live_play: bool) {
        self.state.backboard.begin_sample(frame);
        self.state.ceiling_shot.begin_sample(frame);
        self.state.center.begin_sample(frame);
        self.state.double_tap.begin_sample(frame);
        self.state.flick.begin_sample(frame);
        self.state.half_flip.begin_sample(frame);
        self.state.half_volley.begin_sample(frame);
        self.state.musty_flick.begin_sample(frame);
        self.state.one_timer.begin_sample(frame);
        self.state.pass.begin_sample(frame);
        self.state.speed_flip.begin_sample(frame);
        self.state.touch.begin_sample(frame);
        self.state.wall_aerial.begin_sample(frame);
        self.state.wall_aerial_shot.begin_sample(frame);
        self.state.wavedash.begin_sample(frame);
        self.state.whiff.begin_sample(frame);

        if !live_play {
            self.state.center.clear_current_last();
            self.state.one_timer.clear_current_last();
            self.state.pass.clear_current_last();
            self.state.ceiling_shot.reset_current_last_event_marker();
            self.state.flick.reset_current_last_event_marker();
            self.state.half_flip.reset_current_last_event_marker();
            self.state.musty_flick.reset_current_last_event_marker();
            self.state.speed_flip.reset_current_last_event_marker();
            self.state.wall_aerial.reset_current_last_event_marker();
            self.state
                .wall_aerial_shot
                .reset_current_last_event_marker();
            self.state.wavedash.reset_current_last_event_marker();
        }
    }

    fn finish_sample(&mut self) {
        self.state.center.finish_sample();
        self.state.double_tap.finish_sample();
        self.state.one_timer.finish_sample();
        self.state.pass.finish_sample();
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

    fn project_frame(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let live_play = ctx.get::<LivePlayState>()?.is_live_play;
        let should_sample_powerslide =
            self.last_powerslide_sample_frame != Some(frame.frame_number);
        self.begin_sample(frame, live_play);

        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        for event in Self::events_since(
            &mut self.cursors.core_player,
            match_stats.core_player_events(),
        ) {
            let previous_team_stats = self.state.core.team_stats_for_side(event.is_team_0);
            self.state.core.apply_player_event(event);
            let current_team_stats = self.state.core.team_stats_for_side(event.is_team_0);
            if current_team_stats != previous_team_stats {
                self.state.core_team_events.push(CoreTeamStatsEvent {
                    time: event.time,
                    frame: event.frame,
                    is_team_0: event.is_team_0,
                    delta: core_team_stats_delta(&current_team_stats, &previous_team_stats),
                });
            }
        }

        let backboard = ctx.get::<BackboardCalculator>()?;
        self.state.backboard.apply_events(
            frame,
            Self::events_since(&mut self.cursors.backboard, backboard.events()),
        );

        let ceiling_shot = ctx.get::<CeilingShotCalculator>()?;
        for event in Self::events_since(&mut self.cursors.ceiling_shot, ceiling_shot.events()) {
            self.state.ceiling_shot.apply_event(event, frame);
        }
        let wall_aerial = ctx.get::<WallAerialCalculator>()?;
        for event in Self::events_since(&mut self.cursors.wall_aerial, wall_aerial.events()) {
            self.state.wall_aerial.apply_event(event, frame);
        }
        let wall_aerial_shot = ctx.get::<WallAerialShotCalculator>()?;
        for event in Self::events_since(
            &mut self.cursors.wall_aerial_shot,
            wall_aerial_shot.events(),
        ) {
            self.state.wall_aerial_shot.apply_event(event, frame);
        }
        let double_tap = ctx.get::<DoubleTapCalculator>()?;
        for event in Self::events_since(&mut self.cursors.double_tap, double_tap.events()) {
            self.state.double_tap.apply_event(frame, event);
        }
        let one_timer = ctx.get::<OneTimerCalculator>()?;
        for event in Self::events_since(&mut self.cursors.one_timer, one_timer.events()) {
            self.state.one_timer.apply_event(frame, event);
        }
        let pass = ctx.get::<PassCalculator>()?;
        for event in Self::events_since(&mut self.cursors.pass, pass.events()) {
            self.state.pass.apply_event(frame, event);
        }
        let fifty_fifty = ctx.get::<FiftyFiftyCalculator>()?;
        for event in Self::events_since(&mut self.cursors.fifty_fifty, fifty_fifty.events()) {
            self.state.fifty_fifty.apply_event(event);
        }
        let possession = ctx.get::<PossessionCalculator>()?;
        for event in Self::events_since(&mut self.cursors.possession, possession.events()) {
            self.state.possession.apply_event(event);
        }
        let pressure = ctx.get::<PressureCalculator>()?;
        for event in Self::events_since(&mut self.cursors.pressure, pressure.events()) {
            self.state.pressure.apply_event(event);
        }
        let territorial_pressure = ctx.get::<TerritorialPressureCalculator>()?;
        for event in Self::events_since(
            &mut self.cursors.territorial_pressure_stats,
            territorial_pressure.stats_events(),
        ) {
            self.state.territorial_pressure.apply_event(event);
        }
        let rotation = ctx.get::<RotationCalculator>()?;
        for event in Self::events_since(&mut self.cursors.rotation_player, rotation.player_events())
        {
            self.state.rotation.apply_player_event(event);
        }
        for event in Self::events_since(&mut self.cursors.rotation_team, rotation.team_events()) {
            self.state.rotation.apply_team_event(event);
        }
        let rush = ctx.get::<RushCalculator>()?;
        for event in Self::events_since(&mut self.cursors.rush, rush.events()) {
            self.state.rush.apply_event(event);
        }
        let touch = ctx.get::<TouchCalculator>()?;
        for event in Self::events_since(&mut self.cursors.touch, touch.events()) {
            self.state.touch.apply_touch_event(event, frame);
        }
        for event in Self::events_since(
            &mut self.cursors.touch_ball_movement,
            touch.ball_movement_events(),
        ) {
            self.state.touch.apply_ball_movement_event(event);
        }
        let whiff = ctx.get::<WhiffCalculator>()?;
        for event in Self::events_since(&mut self.cursors.whiff, whiff.events()) {
            self.state.whiff.apply_event(event, frame);
        }
        let wavedash = ctx.get::<WavedashCalculator>()?;
        for event in Self::events_since(&mut self.cursors.wavedash, wavedash.events()) {
            self.state.wavedash.apply_event(event);
        }
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        for event in Self::events_since(&mut self.cursors.speed_flip, speed_flip.events()) {
            self.state.speed_flip.apply_event(event);
        }
        let half_flip = ctx.get::<HalfFlipCalculator>()?;
        for event in Self::events_since(&mut self.cursors.half_flip, half_flip.events()) {
            self.state.half_flip.apply_event(event);
        }
        let flick = ctx.get::<FlickCalculator>()?;
        for event in Self::events_since(&mut self.cursors.flick, flick.events()) {
            self.state.flick.apply_event(event, frame);
        }
        let musty_flick = ctx.get::<MustyFlickCalculator>()?;
        for event in Self::events_since(&mut self.cursors.musty_flick, musty_flick.events()) {
            self.state.musty_flick.apply_event(event, frame);
        }
        let dodge_reset = ctx.get::<DodgeResetCalculator>()?;
        for event in Self::events_since(&mut self.cursors.dodge_reset, dodge_reset.events()) {
            self.state.dodge_reset.apply_event(event);
        }
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        for event in Self::events_since(&mut self.cursors.ball_carry, ball_carry.carry_events()) {
            self.state.ball_carry.apply_event(event);
        }
        let boost = ctx.get::<BoostCalculator>()?;
        for event in Self::events_since(&mut self.cursors.boost_stats, boost.stats_events()) {
            self.state.boost.apply_event(event);
        }
        let bump = ctx.get::<BumpCalculator>()?;
        for event in Self::events_since(&mut self.cursors.bump, bump.events()) {
            self.state.bump.apply_event(event);
        }
        let half_volley = ctx.get::<HalfVolleyCalculator>()?;
        for event in Self::events_since(&mut self.cursors.half_volley, half_volley.events()) {
            self.state.half_volley.apply_event(event, frame);
        }
        let movement = ctx.get::<MovementCalculator>()?;
        for event in Self::events_since(&mut self.cursors.movement, movement.events()) {
            self.state.movement.apply_event(event);
        }
        let positioning = ctx.get::<PositioningCalculator>()?;
        self.state.positioning.apply_events(Self::events_since(
            &mut self.cursors.positioning,
            positioning.events(),
        ));
        let powerslide = ctx.get::<PowerslideCalculator>()?;
        let powerslide_events =
            Self::events_since(&mut self.cursors.powerslide, powerslide.events());
        if should_sample_powerslide {
            self.powerslide.apply_frame(
                &mut self.state.powerslide,
                frame,
                powerslide_events,
                live_play,
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

        self.finish_sample();
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
            live_play_dependency(),
            match_stats_dependency(),
            backboard_dependency(),
            ceiling_shot_dependency(),
            wall_aerial_dependency(),
            wall_aerial_shot_dependency(),
            double_tap_dependency(),
            one_timer_dependency(),
            pass_dependency(),
            fifty_fifty_dependency(),
            possession_dependency(),
            pressure_dependency(),
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
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.project_frame(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.project_frame(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsProjectionNode::new())
}
