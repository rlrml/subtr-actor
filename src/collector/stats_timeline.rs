use std::collections::BTreeSet;
use std::str::FromStr;

use serde::Serialize;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
}

const CORE_MODULE: &str = "core";
const BACKBOARD_MODULE: &str = "backboard";
const CEILING_SHOT_MODULE: &str = "ceiling_shot";
const DOUBLE_TAP_MODULE: &str = "double_tap";
const FIFTY_FIFTY_MODULE: &str = "fifty_fifty";
const POSSESSION_MODULE: &str = "possession";
const PRESSURE_MODULE: &str = "pressure";
const RUSH_MODULE: &str = "rush";
const TOUCH_MODULE: &str = "touch";
const SPEED_FLIP_MODULE: &str = "speed_flip";
const MUSTY_FLICK_MODULE: &str = "musty_flick";
const DODGE_RESET_MODULE: &str = "dodge_reset";
const BALL_CARRY_MODULE: &str = "ball_carry";
const BOOST_MODULE: &str = "boost";
const MOVEMENT_MODULE: &str = "movement";
const POSITIONING_MODULE: &str = "positioning";
const POWERSLIDE_MODULE: &str = "powerslide";
const DEMO_MODULE: &str = "demo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatsTimelineModule {
    Core,
    Backboard,
    CeilingShot,
    DoubleTap,
    FiftyFifty,
    Possession,
    Pressure,
    Rush,
    Touch,
    SpeedFlip,
    MustyFlick,
    DodgeReset,
    BallCarry,
    Boost,
    Movement,
    Positioning,
    Powerslide,
    Demo,
}

impl StatsTimelineModule {
    pub const fn as_str(self) -> &'static str {
        match self {
            StatsTimelineModule::Core => CORE_MODULE,
            StatsTimelineModule::Backboard => BACKBOARD_MODULE,
            StatsTimelineModule::CeilingShot => CEILING_SHOT_MODULE,
            StatsTimelineModule::DoubleTap => DOUBLE_TAP_MODULE,
            StatsTimelineModule::FiftyFifty => FIFTY_FIFTY_MODULE,
            StatsTimelineModule::Possession => POSSESSION_MODULE,
            StatsTimelineModule::Pressure => PRESSURE_MODULE,
            StatsTimelineModule::Rush => RUSH_MODULE,
            StatsTimelineModule::Touch => TOUCH_MODULE,
            StatsTimelineModule::SpeedFlip => SPEED_FLIP_MODULE,
            StatsTimelineModule::MustyFlick => MUSTY_FLICK_MODULE,
            StatsTimelineModule::DodgeReset => DODGE_RESET_MODULE,
            StatsTimelineModule::BallCarry => BALL_CARRY_MODULE,
            StatsTimelineModule::Boost => BOOST_MODULE,
            StatsTimelineModule::Movement => MOVEMENT_MODULE,
            StatsTimelineModule::Positioning => POSITIONING_MODULE,
            StatsTimelineModule::Powerslide => POWERSLIDE_MODULE,
            StatsTimelineModule::Demo => DEMO_MODULE,
        }
    }

    pub fn all_names() -> &'static [&'static str] {
        builtin_stats_module_names()
    }
}

impl FromStr for StatsTimelineModule {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            CORE_MODULE => Ok(Self::Core),
            BACKBOARD_MODULE => Ok(Self::Backboard),
            CEILING_SHOT_MODULE => Ok(Self::CeilingShot),
            DOUBLE_TAP_MODULE => Ok(Self::DoubleTap),
            FIFTY_FIFTY_MODULE => Ok(Self::FiftyFifty),
            POSSESSION_MODULE => Ok(Self::Possession),
            PRESSURE_MODULE => Ok(Self::Pressure),
            RUSH_MODULE => Ok(Self::Rush),
            TOUCH_MODULE => Ok(Self::Touch),
            SPEED_FLIP_MODULE => Ok(Self::SpeedFlip),
            MUSTY_FLICK_MODULE => Ok(Self::MustyFlick),
            DODGE_RESET_MODULE => Ok(Self::DodgeReset),
            BALL_CARRY_MODULE => Ok(Self::BallCarry),
            BOOST_MODULE => Ok(Self::Boost),
            MOVEMENT_MODULE => Ok(Self::Movement),
            POSITIONING_MODULE => Ok(Self::Positioning),
            POWERSLIDE_MODULE => Ok(Self::Powerslide),
            DEMO_MODULE => Ok(Self::Demo),
            invalid => Err(format!(
                "Unknown stats timeline module '{invalid}'. Expected one of: {}",
                Self::all_names().join(", ")
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsTimelineModules {
    processed: BTreeSet<String>,
    emitted: BTreeSet<String>,
    required_derived_signals: Vec<DerivedSignalId>,
}

impl Default for StatsTimelineModules {
    fn default() -> Self {
        Self::all()
    }
}

impl StatsTimelineModules {
    pub fn all_names() -> &'static [&'static str] {
        builtin_stats_module_names()
    }

    pub fn all() -> Self {
        Self::from_builtin_names(builtin_stats_module_names())
            .expect("builtin stats timeline modules should resolve without conflicts")
    }

    pub fn from_builtin_names<I, S>(modules: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut factories = Vec::new();
        for module in modules {
            let module = module.as_ref();
            factories.push(builtin_stats_module_factory_by_name(module).ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                    module.to_owned(),
                ))
            })?);
        }
        Ok(Self::from_resolved(
            crate::collector::stats::resolve_stats_module_factories(factories)?,
        ))
    }

    fn from_resolved(resolved: Vec<crate::collector::stats::ResolvedStatsModuleFactory>) -> Self {
        let mut processed = BTreeSet::new();
        let mut emitted = BTreeSet::new();
        let mut required_derived_signals = BTreeSet::new();

        for resolved_module in resolved {
            processed.insert(resolved_module.name.to_owned());
            if resolved_module.emit {
                emitted.insert(resolved_module.name.to_owned());
            }
            required_derived_signals.extend(resolved_module.factory.required_derived_signals());
        }

        Self {
            processed,
            emitted,
            required_derived_signals: required_derived_signals.into_iter().collect(),
        }
    }

    pub fn contains_name(&self, module_name: &str) -> bool {
        self.processed.contains(module_name)
    }

    pub fn contains(&self, module: StatsTimelineModule) -> bool {
        self.contains_name(module.as_str())
    }

    pub fn emits_name(&self, module_name: &str) -> bool {
        self.emitted.contains(module_name)
    }

    pub fn emits(&self, module: StatsTimelineModule) -> bool {
        self.emits_name(module.as_str())
    }

    pub fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        self.required_derived_signals.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub backboard_events: Vec<BackboardBounceEvent>,
    pub ceiling_shot_events: Vec<CeilingShotEvent>,
    pub double_tap_events: Vec<DoubleTapEvent>,
    pub fifty_fifty_events: Vec<FiftyFiftyEvent>,
    pub rush_events: Vec<RushEvent>,
    pub speed_flip_events: Vec<SpeedFlipEvent>,
    pub frames: Vec<ReplayStatsFrame>,
}

impl ReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&ReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub backboard_events: Vec<BackboardBounceEvent>,
    pub ceiling_shot_events: Vec<CeilingShotEvent>,
    pub double_tap_events: Vec<DoubleTapEvent>,
    pub fifty_fifty_events: Vec<FiftyFiftyEvent>,
    pub rush_events: Vec<RushEvent>,
    pub speed_flip_events: Vec<SpeedFlipEvent>,
    pub frames: Vec<DynamicReplayStatsFrame>,
}

impl DynamicReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&DynamicReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub fifty_fifty: FiftyFiftyStats,
    pub possession: PossessionStats,
    pub pressure: PressureStats,
    pub rush: RushStats,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub fifty_fifty: Vec<ExportedStat>,
    pub possession: Vec<ExportedStat>,
    pub pressure: Vec<ExportedStat>,
    pub rush: Vec<ExportedStat>,
    pub team_zero: DynamicTeamStatsSnapshot,
    pub team_one: DynamicTeamStatsSnapshot,
    pub players: Vec<DynamicPlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamStatsSnapshot {
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicTeamStatsSnapshot {
    pub stats: Vec<ExportedStat>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerStatsSnapshot {
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub core: CorePlayerStats,
    pub backboard: BackboardPlayerStats,
    pub ceiling_shot: CeilingShotStats,
    pub double_tap: DoubleTapPlayerStats,
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub touch: TouchStats,
    pub musty_flick: MustyFlickStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicPlayerStatsSnapshot {
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub stats: Vec<ExportedStat>,
}

impl StatFieldProvider for TeamStatsSnapshot {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.core.visit_stat_fields(visitor);
        self.backboard.visit_stat_fields(visitor);
        self.double_tap.visit_stat_fields(visitor);
        self.ball_carry.visit_stat_fields(visitor);
        self.boost.visit_stat_fields(visitor);
        self.movement.visit_stat_fields(visitor);
        self.powerslide.visit_stat_fields(visitor);
        self.demo.visit_stat_fields(visitor);
    }
}

impl TeamStatsSnapshot {
    fn stat_fields_for_modules(&self, modules: &StatsTimelineModules) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        if modules.emits(StatsTimelineModule::Core) {
            self.core.visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Backboard) {
            self.backboard
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::DoubleTap) {
            self.double_tap
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::BallCarry) {
            self.ball_carry
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Boost) {
            self.boost
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Movement) {
            self.movement
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Powerslide) {
            self.powerslide
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Demo) {
            self.demo.visit_stat_fields(&mut |field| fields.push(field));
        }
        fields
    }
}

impl StatFieldProvider for PlayerStatsSnapshot {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.core.visit_stat_fields(visitor);
        self.backboard.visit_stat_fields(visitor);
        self.ceiling_shot.visit_stat_fields(visitor);
        self.double_tap.visit_stat_fields(visitor);
        self.fifty_fifty.visit_stat_fields(visitor);
        self.speed_flip.visit_stat_fields(visitor);
        self.touch.visit_stat_fields(visitor);
        self.musty_flick.visit_stat_fields(visitor);
        self.dodge_reset.visit_stat_fields(visitor);
        self.ball_carry.visit_stat_fields(visitor);
        self.boost.visit_stat_fields(visitor);
        self.movement.visit_stat_fields(visitor);
        self.positioning.visit_stat_fields(visitor);
        self.powerslide.visit_stat_fields(visitor);
        self.demo.visit_stat_fields(visitor);
    }
}

impl PlayerStatsSnapshot {
    fn stat_fields_for_modules(&self, modules: &StatsTimelineModules) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        if modules.emits(StatsTimelineModule::Core) {
            self.core.visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Backboard) {
            self.backboard
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::CeilingShot) {
            self.ceiling_shot
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::DoubleTap) {
            self.double_tap
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::FiftyFifty) {
            self.fifty_fifty
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::SpeedFlip) {
            self.speed_flip
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Touch) {
            self.touch
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::MustyFlick) {
            self.musty_flick
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::DodgeReset) {
            self.dodge_reset
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::BallCarry) {
            self.ball_carry
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Boost) {
            self.boost
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Movement) {
            self.movement
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Positioning) {
            self.positioning
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Powerslide) {
            self.powerslide
                .visit_stat_fields(&mut |field| fields.push(field));
        }
        if modules.emits(StatsTimelineModule::Demo) {
            self.demo.visit_stat_fields(&mut |field| fields.push(field));
        }
        fields
    }
}

impl ReplayStatsFrame {
    pub fn into_dynamic(self) -> DynamicReplayStatsFrame {
        self.into_dynamic_with_modules(&StatsTimelineModules::all())
    }

    fn into_dynamic_with_modules(self, modules: &StatsTimelineModules) -> DynamicReplayStatsFrame {
        DynamicReplayStatsFrame {
            frame_number: self.frame_number,
            time: self.time,
            dt: self.dt,
            seconds_remaining: self.seconds_remaining,
            game_state: self.game_state,
            is_live_play: self.is_live_play,
            fifty_fifty: if modules.emits(StatsTimelineModule::FiftyFifty) {
                self.fifty_fifty.stat_fields()
            } else {
                Vec::new()
            },
            possession: if modules.emits(StatsTimelineModule::Possession) {
                self.possession.stat_fields()
            } else {
                Vec::new()
            },
            pressure: if modules.emits(StatsTimelineModule::Pressure) {
                self.pressure.stat_fields()
            } else {
                Vec::new()
            },
            rush: if modules.emits(StatsTimelineModule::Rush) {
                self.rush.stat_fields()
            } else {
                Vec::new()
            },
            team_zero: DynamicTeamStatsSnapshot {
                stats: self.team_zero.stat_fields_for_modules(modules),
            },
            team_one: DynamicTeamStatsSnapshot {
                stats: self.team_one.stat_fields_for_modules(modules),
            },
            players: self
                .players
                .into_iter()
                .map(|player| {
                    let stats = player.stat_fields_for_modules(modules);
                    DynamicPlayerStatsSnapshot {
                        player_id: player.player_id,
                        name: player.name,
                        is_team_0: player.is_team_0,
                        stats,
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct StatsTimelineReducers {
    backboard: BackboardReducer,
    ceiling_shot: CeilingShotReducer,
    double_tap: DoubleTapReducer,
    fifty_fifty: FiftyFiftyReducer,
    possession: PossessionReducer,
    pressure: PressureReducer,
    rush: RushReducer,
    match_stats: MatchStatsReducer,
    touch: TouchReducer,
    speed_flip: SpeedFlipReducer,
    musty_flick: MustyFlickReducer,
    ball_carry: BallCarryReducer,
    boost: BoostReducer,
    movement: MovementReducer,
    positioning: PositioningReducer,
    powerslide: PowerslideReducer,
    demo: DemoReducer,
    dodge_reset: DodgeResetReducer,
}

impl StatsTimelineReducers {
    fn with_positioning_config(config: PositioningReducerConfig) -> Self {
        Self {
            positioning: PositioningReducer::with_config(config),
            ..Self::default()
        }
    }

    fn with_pressure_config(config: PressureReducerConfig) -> Self {
        Self {
            pressure: PressureReducer::with_config(config),
            ..Self::default()
        }
    }

    fn with_rush_config(config: RushReducerConfig) -> Self {
        Self {
            rush: RushReducer::with_config(config),
            ..Self::default()
        }
    }

    fn on_replay_meta(
        &mut self,
        modules: &StatsTimelineModules,
        meta: &ReplayMeta,
    ) -> SubtrActorResult<()> {
        if modules.contains(StatsTimelineModule::Backboard) {
            self.backboard.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::CeilingShot) {
            self.ceiling_shot.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::DoubleTap) {
            self.double_tap.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Possession) {
            self.possession.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Pressure) {
            self.pressure.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Rush) {
            self.rush.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Core) {
            self.match_stats.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::FiftyFifty) {
            self.fifty_fifty.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Touch) {
            self.touch.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::SpeedFlip) {
            self.speed_flip.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::MustyFlick) {
            self.musty_flick.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::BallCarry) {
            self.ball_carry.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Boost) {
            self.boost.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Movement) {
            self.movement.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Positioning) {
            self.positioning.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Powerslide) {
            self.powerslide.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::Demo) {
            self.demo.on_replay_meta(meta)?;
        }
        if modules.contains(StatsTimelineModule::DodgeReset) {
            self.dodge_reset.on_replay_meta(meta)?;
        }
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        modules: &StatsTimelineModules,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        if modules.contains(StatsTimelineModule::Backboard) {
            self.backboard.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::CeilingShot) {
            self.ceiling_shot.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::DoubleTap) {
            self.double_tap.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Possession) {
            self.possession.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Pressure) {
            self.pressure.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Rush) {
            self.rush.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Core) {
            self.match_stats.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::FiftyFifty) {
            self.fifty_fifty.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Touch) {
            self.touch.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::SpeedFlip) {
            self.speed_flip.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::MustyFlick) {
            self.musty_flick.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::BallCarry) {
            self.ball_carry.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Boost) {
            self.boost.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Movement) {
            self.movement.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Positioning) {
            self.positioning.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Powerslide) {
            self.powerslide.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::Demo) {
            self.demo.on_sample_with_context(sample, ctx)?;
        }
        if modules.contains(StatsTimelineModule::DodgeReset) {
            self.dodge_reset.on_sample_with_context(sample, ctx)?;
        }
        Ok(())
    }

    fn finish(&mut self, modules: &StatsTimelineModules) -> SubtrActorResult<()> {
        if modules.contains(StatsTimelineModule::CeilingShot) {
            self.ceiling_shot.finish()?;
        }
        if modules.contains(StatsTimelineModule::Possession) {
            self.possession.finish()?;
        }
        if modules.contains(StatsTimelineModule::Pressure) {
            self.pressure.finish()?;
        }
        if modules.contains(StatsTimelineModule::Rush) {
            self.rush.finish()?;
        }
        if modules.contains(StatsTimelineModule::Core) {
            self.match_stats.finish()?;
        }
        if modules.contains(StatsTimelineModule::FiftyFifty) {
            self.fifty_fifty.finish()?;
        }
        if modules.contains(StatsTimelineModule::Touch) {
            self.touch.finish()?;
        }
        if modules.contains(StatsTimelineModule::SpeedFlip) {
            self.speed_flip.finish()?;
        }
        if modules.contains(StatsTimelineModule::MustyFlick) {
            self.musty_flick.finish()?;
        }
        if modules.contains(StatsTimelineModule::BallCarry) {
            self.ball_carry.finish()?;
        }
        if modules.contains(StatsTimelineModule::Boost) {
            self.boost.finish()?;
        }
        if modules.contains(StatsTimelineModule::Movement) {
            self.movement.finish()?;
        }
        if modules.contains(StatsTimelineModule::Positioning) {
            self.positioning.finish()?;
        }
        if modules.contains(StatsTimelineModule::Powerslide) {
            self.powerslide.finish()?;
        }
        if modules.contains(StatsTimelineModule::Demo) {
            self.demo.finish()?;
        }
        if modules.contains(StatsTimelineModule::DodgeReset) {
            self.dodge_reset.finish()?;
        }
        Ok(())
    }
}

pub struct StatsTimelineCollector {
    modules: StatsTimelineModules,
    reducers: StatsTimelineReducers,
    derived_signals: DerivedSignalGraph,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    last_sample_time: Option<f32>,
    last_sample: Option<StatsSample>,
    last_live_play: Option<bool>,
    live_play_tracker: LivePlayTracker,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        let modules = StatsTimelineModules::default();
        Self {
            derived_signals: derived_signal_graph_for_ids(modules.required_derived_signals()),
            modules,
            reducers: StatsTimelineReducers::default(),
            replay_meta: None,
            frames: Vec::new(),
            last_sample_time: None,
            last_sample: None,
            last_live_play: None,
            live_play_tracker: LivePlayTracker::default(),
        }
    }
}

impl StatsTimelineCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn only_modules<I>(modules: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::try_only_modules(modules)
            .expect("builtin stats timeline module names should be valid")
    }

    pub fn try_only_modules<I>(modules: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Ok(Self::default()
            .with_module_selection(StatsTimelineModules::from_builtin_names(modules)?))
    }

    pub fn with_module_selection(mut self, modules: StatsTimelineModules) -> Self {
        self.derived_signals = derived_signal_graph_for_ids(modules.required_derived_signals());
        self.modules = modules;
        self
    }

    pub fn with_positioning_config(config: PositioningReducerConfig) -> Self {
        Self {
            reducers: StatsTimelineReducers::with_positioning_config(config),
            ..Self::default()
        }
    }

    pub fn with_pressure_config(config: PressureReducerConfig) -> Self {
        Self {
            reducers: StatsTimelineReducers::with_pressure_config(config),
            ..Self::default()
        }
    }

    pub fn with_rush_config(config: RushReducerConfig) -> Self {
        Self {
            reducers: StatsTimelineReducers::with_rush_config(config),
            ..Self::default()
        }
    }

    pub fn get_replay_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        Ok(self.into_timeline())
    }

    pub fn get_dynamic_replay_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<DynamicReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        Ok(self.into_dynamic_timeline())
    }

    pub fn into_timeline(self) -> ReplayStatsTimeline {
        let replay_meta = self
            .replay_meta
            .expect("replay metadata should be initialized before building a stats timeline");
        let config = StatsTimelineConfig {
            most_back_forward_threshold_y: self
                .reducers
                .positioning
                .config()
                .most_back_forward_threshold_y,
            pressure_neutral_zone_half_width_y: self
                .reducers
                .pressure
                .config()
                .neutral_zone_half_width_y,
            rush_max_start_y: self.reducers.rush.config().max_start_y,
            rush_attack_support_distance_y: self.reducers.rush.config().attack_support_distance_y,
            rush_defender_distance_y: self.reducers.rush.config().defender_distance_y,
            rush_min_possession_retained_seconds: self
                .reducers
                .rush
                .config()
                .min_possession_retained_seconds,
        };
        let mut timeline_events = Vec::new();
        if self.modules.emits(StatsTimelineModule::Core) {
            timeline_events.extend(self.reducers.match_stats.timeline().iter().cloned());
        }
        if self.modules.emits(StatsTimelineModule::Demo) {
            timeline_events.extend(self.reducers.demo.timeline().iter().cloned());
        }
        timeline_events.sort_by(|left, right| left.time.total_cmp(&right.time));
        ReplayStatsTimeline {
            config,
            replay_meta,
            timeline_events,
            backboard_events: if self.modules.emits(StatsTimelineModule::Backboard) {
                self.reducers.backboard.events().to_vec()
            } else {
                Vec::new()
            },
            ceiling_shot_events: if self.modules.emits(StatsTimelineModule::CeilingShot) {
                self.reducers.ceiling_shot.events().to_vec()
            } else {
                Vec::new()
            },
            double_tap_events: if self.modules.emits(StatsTimelineModule::DoubleTap) {
                self.reducers.double_tap.events().to_vec()
            } else {
                Vec::new()
            },
            fifty_fifty_events: if self.modules.emits(StatsTimelineModule::FiftyFifty) {
                self.reducers.fifty_fifty.events().to_vec()
            } else {
                Vec::new()
            },
            rush_events: if self.modules.emits(StatsTimelineModule::Rush) {
                self.reducers.rush.events().to_vec()
            } else {
                Vec::new()
            },
            speed_flip_events: if self.modules.emits(StatsTimelineModule::SpeedFlip) {
                self.reducers.speed_flip.events().to_vec()
            } else {
                Vec::new()
            },
            frames: self.frames,
        }
    }

    pub fn into_dynamic_timeline(self) -> DynamicReplayStatsTimeline {
        let replay_meta = self
            .replay_meta
            .expect("replay metadata should be initialized before building a stats timeline");
        let config = StatsTimelineConfig {
            most_back_forward_threshold_y: self
                .reducers
                .positioning
                .config()
                .most_back_forward_threshold_y,
            pressure_neutral_zone_half_width_y: self
                .reducers
                .pressure
                .config()
                .neutral_zone_half_width_y,
            rush_max_start_y: self.reducers.rush.config().max_start_y,
            rush_attack_support_distance_y: self.reducers.rush.config().attack_support_distance_y,
            rush_defender_distance_y: self.reducers.rush.config().defender_distance_y,
            rush_min_possession_retained_seconds: self
                .reducers
                .rush
                .config()
                .min_possession_retained_seconds,
        };
        let mut timeline_events = Vec::new();
        if self.modules.emits(StatsTimelineModule::Core) {
            timeline_events.extend(self.reducers.match_stats.timeline().iter().cloned());
        }
        if self.modules.emits(StatsTimelineModule::Demo) {
            timeline_events.extend(self.reducers.demo.timeline().iter().cloned());
        }
        timeline_events.sort_by(|left, right| left.time.total_cmp(&right.time));
        let modules = self.modules.clone();
        DynamicReplayStatsTimeline {
            config,
            replay_meta,
            timeline_events,
            backboard_events: if modules.emits(StatsTimelineModule::Backboard) {
                self.reducers.backboard.events().to_vec()
            } else {
                Vec::new()
            },
            ceiling_shot_events: if modules.emits(StatsTimelineModule::CeilingShot) {
                self.reducers.ceiling_shot.events().to_vec()
            } else {
                Vec::new()
            },
            double_tap_events: if modules.emits(StatsTimelineModule::DoubleTap) {
                self.reducers.double_tap.events().to_vec()
            } else {
                Vec::new()
            },
            fifty_fifty_events: if modules.emits(StatsTimelineModule::FiftyFifty) {
                self.reducers.fifty_fifty.events().to_vec()
            } else {
                Vec::new()
            },
            rush_events: if modules.emits(StatsTimelineModule::Rush) {
                self.reducers.rush.events().to_vec()
            } else {
                Vec::new()
            },
            speed_flip_events: if modules.emits(StatsTimelineModule::SpeedFlip) {
                self.reducers.speed_flip.events().to_vec()
            } else {
                Vec::new()
            },
            frames: self
                .frames
                .into_iter()
                .map(|frame| frame.into_dynamic_with_modules(&modules))
                .collect(),
        }
    }

    fn snapshot_frame(
        &self,
        sample: &StatsSample,
        replay_meta: &ReplayMeta,
        live_play: bool,
    ) -> ReplayStatsFrame {
        ReplayStatsFrame {
            frame_number: sample.frame_number,
            time: sample.time,
            dt: sample.dt,
            seconds_remaining: sample.seconds_remaining,
            game_state: sample.game_state,
            is_live_play: live_play,
            fifty_fifty: if self.modules.emits(StatsTimelineModule::FiftyFifty) {
                self.reducers.fifty_fifty.stats().clone()
            } else {
                FiftyFiftyStats::default()
            },
            possession: if self.modules.emits(StatsTimelineModule::Possession) {
                self.reducers.possession.stats().clone()
            } else {
                PossessionStats::default()
            },
            pressure: if self.modules.emits(StatsTimelineModule::Pressure) {
                self.reducers.pressure.stats().clone()
            } else {
                PressureStats::default()
            },
            rush: if self.modules.emits(StatsTimelineModule::Rush) {
                self.reducers.rush.stats().clone()
            } else {
                RushStats::default()
            },
            team_zero: TeamStatsSnapshot {
                core: if self.modules.emits(StatsTimelineModule::Core) {
                    self.reducers.match_stats.team_zero_stats()
                } else {
                    CoreTeamStats::default()
                },
                backboard: if self.modules.emits(StatsTimelineModule::Backboard) {
                    self.reducers.backboard.team_zero_stats().clone()
                } else {
                    BackboardTeamStats::default()
                },
                double_tap: if self.modules.emits(StatsTimelineModule::DoubleTap) {
                    self.reducers.double_tap.team_zero_stats().clone()
                } else {
                    DoubleTapTeamStats::default()
                },
                ball_carry: if self.modules.emits(StatsTimelineModule::BallCarry) {
                    self.reducers.ball_carry.team_zero_stats().clone()
                } else {
                    BallCarryStats::default()
                },
                boost: if self.modules.emits(StatsTimelineModule::Boost) {
                    self.reducers.boost.team_zero_stats().clone()
                } else {
                    BoostStats::default()
                },
                movement: if self.modules.emits(StatsTimelineModule::Movement) {
                    self.reducers.movement.team_zero_stats().clone()
                } else {
                    MovementStats::default()
                },
                powerslide: if self.modules.emits(StatsTimelineModule::Powerslide) {
                    self.reducers.powerslide.team_zero_stats().clone()
                } else {
                    PowerslideStats::default()
                },
                demo: if self.modules.emits(StatsTimelineModule::Demo) {
                    self.reducers.demo.team_zero_stats().clone()
                } else {
                    DemoTeamStats::default()
                },
            },
            team_one: TeamStatsSnapshot {
                core: if self.modules.emits(StatsTimelineModule::Core) {
                    self.reducers.match_stats.team_one_stats()
                } else {
                    CoreTeamStats::default()
                },
                backboard: if self.modules.emits(StatsTimelineModule::Backboard) {
                    self.reducers.backboard.team_one_stats().clone()
                } else {
                    BackboardTeamStats::default()
                },
                double_tap: if self.modules.emits(StatsTimelineModule::DoubleTap) {
                    self.reducers.double_tap.team_one_stats().clone()
                } else {
                    DoubleTapTeamStats::default()
                },
                ball_carry: if self.modules.emits(StatsTimelineModule::BallCarry) {
                    self.reducers.ball_carry.team_one_stats().clone()
                } else {
                    BallCarryStats::default()
                },
                boost: if self.modules.emits(StatsTimelineModule::Boost) {
                    self.reducers.boost.team_one_stats().clone()
                } else {
                    BoostStats::default()
                },
                movement: if self.modules.emits(StatsTimelineModule::Movement) {
                    self.reducers.movement.team_one_stats().clone()
                } else {
                    MovementStats::default()
                },
                powerslide: if self.modules.emits(StatsTimelineModule::Powerslide) {
                    self.reducers.powerslide.team_one_stats().clone()
                } else {
                    PowerslideStats::default()
                },
                demo: if self.modules.emits(StatsTimelineModule::Demo) {
                    self.reducers.demo.team_one_stats().clone()
                } else {
                    DemoTeamStats::default()
                },
            },
            players: replay_meta
                .player_order()
                .map(|player| PlayerStatsSnapshot {
                    player_id: player.remote_id.clone(),
                    name: player.name.clone(),
                    is_team_0: replay_meta
                        .team_zero
                        .iter()
                        .any(|team_player| team_player.remote_id == player.remote_id),
                    core: if self.modules.emits(StatsTimelineModule::Core) {
                        self.reducers
                            .match_stats
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        CorePlayerStats::default()
                    },
                    backboard: if self.modules.emits(StatsTimelineModule::Backboard) {
                        self.reducers
                            .backboard
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        BackboardPlayerStats::default()
                    },
                    ceiling_shot: if self.modules.emits(StatsTimelineModule::CeilingShot) {
                        self.reducers
                            .ceiling_shot
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        CeilingShotStats::default()
                    },
                    double_tap: if self.modules.emits(StatsTimelineModule::DoubleTap) {
                        self.reducers
                            .double_tap
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        DoubleTapPlayerStats::default()
                    },
                    fifty_fifty: if self.modules.emits(StatsTimelineModule::FiftyFifty) {
                        self.reducers
                            .fifty_fifty
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        FiftyFiftyPlayerStats::default()
                    },
                    speed_flip: if self.modules.emits(StatsTimelineModule::SpeedFlip) {
                        self.reducers
                            .speed_flip
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        SpeedFlipStats::default()
                    },
                    touch: if self.modules.emits(StatsTimelineModule::Touch) {
                        self.reducers
                            .touch
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                            .with_complete_labeled_touch_counts()
                    } else {
                        TouchStats::default()
                    },
                    musty_flick: if self.modules.emits(StatsTimelineModule::MustyFlick) {
                        self.reducers
                            .musty_flick
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        MustyFlickStats::default()
                    },
                    dodge_reset: if self.modules.emits(StatsTimelineModule::DodgeReset) {
                        self.reducers
                            .dodge_reset
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        DodgeResetStats::default()
                    },
                    ball_carry: if self.modules.emits(StatsTimelineModule::BallCarry) {
                        self.reducers
                            .ball_carry
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        BallCarryStats::default()
                    },
                    boost: if self.modules.emits(StatsTimelineModule::Boost) {
                        self.reducers
                            .boost
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        BoostStats::default()
                    },
                    movement: if self.modules.emits(StatsTimelineModule::Movement) {
                        self.reducers
                            .movement
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                            .with_complete_labeled_tracked_time()
                    } else {
                        MovementStats::default()
                    },
                    positioning: if self.modules.emits(StatsTimelineModule::Positioning) {
                        self.reducers
                            .positioning
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        PositioningStats::default()
                    },
                    powerslide: if self.modules.emits(StatsTimelineModule::Powerslide) {
                        self.reducers
                            .powerslide
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        PowerslideStats::default()
                    },
                    demo: if self.modules.emits(StatsTimelineModule::Demo) {
                        self.reducers
                            .demo
                            .player_stats()
                            .get(&player.remote_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        DemoPlayerStats::default()
                    },
                })
                .collect(),
        }
    }
}

impl Collector for StatsTimelineCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if self.replay_meta.is_none() {
            let replay_meta = processor.get_replay_meta()?;
            self.derived_signals.on_replay_meta(&replay_meta)?;
            self.reducers.on_replay_meta(&self.modules, &replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let sample = StatsSample::from_processor(processor, frame_number, current_time, dt)?;
        let live_play = self.live_play_tracker.is_live_play(&sample);
        let analysis_context = self.derived_signals.evaluate(&sample)?;
        self.reducers
            .on_sample_with_context(&self.modules, &sample, analysis_context)?;
        self.last_sample_time = Some(current_time);
        self.last_live_play = Some(live_play);

        let replay_meta = self
            .replay_meta
            .as_ref()
            .expect("replay metadata should be initialized before snapshotting");
        self.frames
            .push(self.snapshot_frame(&sample, replay_meta, live_play));
        self.last_sample = Some(sample);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.derived_signals.finish()?;
        self.reducers.finish(&self.modules)?;
        let Some(last_sample) = self.last_sample.as_ref() else {
            return Ok(());
        };
        let Some(replay_meta) = self.replay_meta.as_ref() else {
            return Ok(());
        };
        let final_snapshot = self.snapshot_frame(
            last_sample,
            replay_meta,
            self.last_live_play.unwrap_or(false),
        );
        if let Some(last_frame) = self.frames.last_mut() {
            *last_frame = final_snapshot;
        }
        Ok(())
    }
}
