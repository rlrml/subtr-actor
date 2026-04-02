use std::collections::HashMap;
use std::sync::Arc;

use serde::{Serialize, Serializer};

use crate::*;

use super::types::{serialize_to_json_value, StatsModule, StatsModuleFactory};

fn player_stats_entries<'a, T>(
    player_stats: &'a HashMap<PlayerId, T>,
) -> Vec<PlayerStatsEntry<'a, T>> {
    let mut entries: Vec<_> = player_stats
        .iter()
        .map(|(player_id, stats)| PlayerStatsEntry {
            player_id: player_id.clone(),
            stats,
        })
        .collect();
    entries.sort_by(|left, right| {
        format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
    });
    entries
}

#[derive(Serialize)]
struct PlayerStatsEntry<'a, T> {
    player_id: PlayerId,
    stats: &'a T,
}

#[derive(Serialize)]
struct OwnedPlayerStatsEntry<T> {
    player_id: PlayerId,
    stats: T,
}

#[derive(Serialize)]
struct PlayerStatsExport<'a, T> {
    player_stats: Vec<PlayerStatsEntry<'a, T>>,
}

#[derive(Serialize)]
struct OwnedPlayerStatsExport<T> {
    player_stats: Vec<OwnedPlayerStatsEntry<T>>,
}

#[derive(Serialize)]
struct PlayerStatsWithEventsExport<'a, T, E> {
    player_stats: Vec<PlayerStatsEntry<'a, T>>,
    events: &'a [E],
}

#[derive(Serialize)]
struct TeamPlayerStatsExport<'a, Team, Player> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}

#[derive(Serialize)]
struct TeamOwnedPlayerStatsExport<'a, Team, Player> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<OwnedPlayerStatsEntry<Player>>,
}

#[derive(Serialize)]
struct TeamPlayerStatsWithEventsExport<'a, Team, Player, Event> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    events: &'a [Event],
}

#[derive(Serialize)]
struct StatsExport<'a, T> {
    stats: &'a T,
}

#[derive(Serialize)]
struct StatsWithEventsExport<'a, T, E> {
    stats: &'a T,
    events: &'a [E],
}

#[derive(Serialize)]
struct StatsWithPlayerEventsExport<'a, T, Player, E> {
    stats: &'a T,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    events: &'a [E],
}

#[derive(Serialize)]
struct StatsWithPlayerStatsExport<'a, T, Player> {
    stats: &'a T,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}

#[derive(Serialize)]
struct CoreStatsExport<'a> {
    team_zero: CoreTeamStats,
    team_one: CoreTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, CorePlayerStats>>,
    timeline: &'a [TimelineEvent],
}

#[derive(Serialize)]
struct CoreStatsSnapshotExport {
    team_zero: CoreTeamStatsPlayback,
    team_one: CoreTeamStatsPlayback,
    player_stats: Vec<OwnedPlayerStatsEntry<CorePlayerStatsPlayback>>,
}

#[derive(Serialize)]
struct CoreTeamStatsPlayback {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
    counter_attack_goal_count: u32,
    sustained_pressure_goal_count: u32,
    other_buildup_goal_count: u32,
}

impl From<CoreTeamStats> for CoreTeamStatsPlayback {
    fn from(stats: CoreTeamStats) -> Self {
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
            kickoff_goal_count: stats.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: stats.goal_after_kickoff.short_goal_count,
            medium_goal_count: stats.goal_after_kickoff.medium_goal_count,
            long_goal_count: stats.goal_after_kickoff.long_goal_count,
            goal_times: stats.goal_after_kickoff.goal_times().to_vec(),
            counter_attack_goal_count: stats.goal_buildup.counter_attack_goal_count,
            sustained_pressure_goal_count: stats.goal_buildup.sustained_pressure_goal_count,
            other_buildup_goal_count: stats.goal_buildup.other_buildup_goal_count,
        }
    }
}

#[derive(Serialize)]
struct CorePlayerStatsPlayback {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    goals_conceded_while_last_defender: u32,
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
    counter_attack_goal_count: u32,
    sustained_pressure_goal_count: u32,
    other_buildup_goal_count: u32,
}

impl From<&CorePlayerStats> for CorePlayerStatsPlayback {
    fn from(stats: &CorePlayerStats) -> Self {
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
            goals_conceded_while_last_defender: stats.goals_conceded_while_last_defender,
            kickoff_goal_count: stats.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: stats.goal_after_kickoff.short_goal_count,
            medium_goal_count: stats.goal_after_kickoff.medium_goal_count,
            long_goal_count: stats.goal_after_kickoff.long_goal_count,
            goal_times: stats.goal_after_kickoff.goal_times().to_vec(),
            counter_attack_goal_count: stats.goal_buildup.counter_attack_goal_count,
            sustained_pressure_goal_count: stats.goal_buildup.sustained_pressure_goal_count,
            other_buildup_goal_count: stats.goal_buildup.other_buildup_goal_count,
        }
    }
}

#[derive(Serialize)]
struct DemoStatsExport<'a> {
    team_zero: &'a DemoTeamStats,
    team_one: &'a DemoTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, DemoPlayerStats>>,
    timeline: &'a [TimelineEvent],
}

macro_rules! delegate_stats_reducer {
    ($runtime:ty, $field:ident) => {
        impl StatsReducer for $runtime {
            fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
                self.$field.on_replay_meta(meta)
            }

            fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
                self.$field.required_derived_signals()
            }

            fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
                self.$field.on_sample(sample)
            }

            fn on_sample_with_context(
                &mut self,
                sample: &CoreSample,
                ctx: &AnalysisContext,
            ) -> SubtrActorResult<()> {
                self.$field.on_sample_with_context(sample, ctx)
            }

            fn finish(&mut self) -> SubtrActorResult<()> {
                self.$field.finish()
            }
        }
    };
}

macro_rules! player_stats_module {
    ($runtime:ident, $name:literal, $reducer:ty) => {
        struct $runtime {
            reducer: $reducer,
        }

        impl StatsModule for $runtime {
            fn name(&self) -> &'static str {
                $name
            }

            fn playback_frame_json(
                &self,
                _replay_meta: &ReplayMeta,
            ) -> SubtrActorResult<Option<serde_json::Value>> {
                Ok(Some(serialize_to_json_value(&PlayerStatsExport {
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                })?))
            }
        }

        delegate_stats_reducer!($runtime, reducer);

        impl Serialize for $runtime {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                PlayerStatsExport {
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                }
                .serialize(serializer)
            }
        }
    };
}

macro_rules! player_stats_events_module {
    ($runtime:ident, $name:literal, $reducer:ty) => {
        struct $runtime {
            reducer: $reducer,
        }

        impl StatsModule for $runtime {
            fn name(&self) -> &'static str {
                $name
            }

            fn playback_frame_json(
                &self,
                _replay_meta: &ReplayMeta,
            ) -> SubtrActorResult<Option<serde_json::Value>> {
                Ok(Some(serialize_to_json_value(&PlayerStatsExport {
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                })?))
            }
        }

        delegate_stats_reducer!($runtime, reducer);

        impl Serialize for $runtime {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                PlayerStatsWithEventsExport {
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                    events: self.reducer.events(),
                }
                .serialize(serializer)
            }
        }
    };
}

macro_rules! team_player_stats_module {
    ($runtime:ident, $name:literal, $reducer:ty) => {
        struct $runtime {
            reducer: $reducer,
        }

        impl StatsModule for $runtime {
            fn name(&self) -> &'static str {
                $name
            }

            fn playback_frame_json(
                &self,
                _replay_meta: &ReplayMeta,
            ) -> SubtrActorResult<Option<serde_json::Value>> {
                Ok(Some(serialize_to_json_value(&TeamPlayerStatsExport {
                    team_zero: self.reducer.team_zero_stats(),
                    team_one: self.reducer.team_one_stats(),
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                })?))
            }
        }

        delegate_stats_reducer!($runtime, reducer);

        impl Serialize for $runtime {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                TeamPlayerStatsExport {
                    team_zero: self.reducer.team_zero_stats(),
                    team_one: self.reducer.team_one_stats(),
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                }
                .serialize(serializer)
            }
        }
    };
}

macro_rules! team_player_events_module {
    ($runtime:ident, $name:literal, $reducer:ty) => {
        struct $runtime {
            reducer: $reducer,
        }

        impl StatsModule for $runtime {
            fn name(&self) -> &'static str {
                $name
            }

            fn playback_frame_json(
                &self,
                _replay_meta: &ReplayMeta,
            ) -> SubtrActorResult<Option<serde_json::Value>> {
                Ok(Some(serialize_to_json_value(&TeamPlayerStatsExport {
                    team_zero: self.reducer.team_zero_stats(),
                    team_one: self.reducer.team_one_stats(),
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                })?))
            }
        }

        delegate_stats_reducer!($runtime, reducer);

        impl Serialize for $runtime {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                TeamPlayerStatsWithEventsExport {
                    team_zero: self.reducer.team_zero_stats(),
                    team_one: self.reducer.team_one_stats(),
                    player_stats: player_stats_entries(self.reducer.player_stats()),
                    events: self.reducer.events(),
                }
                .serialize(serializer)
            }
        }
    };
}

macro_rules! stats_only_module {
    ($runtime:ident, $name:literal, $reducer:ty) => {
        struct $runtime {
            reducer: $reducer,
        }

        impl StatsModule for $runtime {
            fn name(&self) -> &'static str {
                $name
            }

            fn playback_frame_json(
                &self,
                _replay_meta: &ReplayMeta,
            ) -> SubtrActorResult<Option<serde_json::Value>> {
                Ok(Some(serialize_to_json_value(&StatsExport {
                    stats: self.reducer.stats(),
                })?))
            }
        }

        delegate_stats_reducer!($runtime, reducer);

        impl Serialize for $runtime {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                StatsExport {
                    stats: self.reducer.stats(),
                }
                .serialize(serializer)
            }
        }
    };
}

player_stats_module!(DodgeResetStatsModule, "dodge_reset", DodgeResetReducer);

player_stats_events_module!(CeilingShotStatsModule, "ceiling_shot", CeilingShotReducer);
player_stats_events_module!(SpeedFlipStatsModule, "speed_flip", SpeedFlipReducer);
player_stats_events_module!(MustyFlickStatsModule, "musty_flick", MustyFlickReducer);

team_player_stats_module!(BoostStatsModule, "boost", BoostReducer);
team_player_stats_module!(PowerslideStatsModule, "powerslide", PowerslideReducer);

team_player_events_module!(BackboardStatsModule, "backboard", BackboardReducer);
team_player_events_module!(DoubleTapStatsModule, "double_tap", DoubleTapReducer);

stats_only_module!(PossessionStatsModule, "possession", PossessionReducer);

struct TouchStatsModule {
    reducer: TouchReducer,
}

impl StatsModule for TouchStatsModule {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        let player_stats = self
            .reducer
            .player_stats()
            .iter()
            .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                player_id: player_id.clone(),
                stats: stats.clone().with_complete_labeled_touch_counts(),
            })
            .collect();
        Ok(Some(serialize_to_json_value(&OwnedPlayerStatsExport {
            player_stats,
        })?))
    }
}

delegate_stats_reducer!(TouchStatsModule, reducer);

impl Serialize for TouchStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PlayerStatsExport {
            player_stats: player_stats_entries(self.reducer.player_stats()),
        }
        .serialize(serializer)
    }
}

struct MovementStatsModule {
    reducer: MovementReducer,
}

impl StatsModule for MovementStatsModule {
    fn name(&self) -> &'static str {
        "movement"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        let player_stats = self
            .reducer
            .player_stats()
            .iter()
            .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                player_id: player_id.clone(),
                stats: stats.clone().with_complete_labeled_tracked_time(),
            })
            .collect();
        Ok(Some(serialize_to_json_value(
            &TeamOwnedPlayerStatsExport {
                team_zero: self.reducer.team_zero_stats(),
                team_one: self.reducer.team_one_stats(),
                player_stats,
            },
        )?))
    }
}

delegate_stats_reducer!(MovementStatsModule, reducer);

impl Serialize for MovementStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TeamPlayerStatsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
        }
        .serialize(serializer)
    }
}

struct PositioningStatsModule {
    reducer: PositioningReducer,
}

impl StatsModule for PositioningStatsModule {
    fn name(&self) -> &'static str {
        "positioning"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&PlayerStatsExport {
            player_stats: player_stats_entries(self.reducer.player_stats()),
        })?))
    }

    fn playback_config_json(&self) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&serde_json::json!({
            "most_back_forward_threshold_y": self.reducer.config().most_back_forward_threshold_y,
        }))?))
    }
}

delegate_stats_reducer!(PositioningStatsModule, reducer);

impl Serialize for PositioningStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PlayerStatsExport {
            player_stats: player_stats_entries(self.reducer.player_stats()),
        }
        .serialize(serializer)
    }
}

struct PressureStatsModule {
    reducer: PressureReducer,
}

impl StatsModule for PressureStatsModule {
    fn name(&self) -> &'static str {
        "pressure"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&StatsExport {
            stats: self.reducer.stats(),
        })?))
    }

    fn playback_config_json(&self) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&serde_json::json!({
            "pressure_neutral_zone_half_width_y": self.reducer.config().neutral_zone_half_width_y,
        }))?))
    }
}

delegate_stats_reducer!(PressureStatsModule, reducer);

impl Serialize for PressureStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        StatsExport {
            stats: self.reducer.stats(),
        }
        .serialize(serializer)
    }
}

struct RushStatsModule {
    reducer: RushReducer,
}

impl StatsModule for RushStatsModule {
    fn name(&self) -> &'static str {
        "rush"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&StatsExport {
            stats: self.reducer.stats(),
        })?))
    }

    fn playback_config_json(&self) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&serde_json::json!({
            "rush_max_start_y": self.reducer.config().max_start_y,
            "rush_attack_support_distance_y": self.reducer.config().attack_support_distance_y,
            "rush_defender_distance_y": self.reducer.config().defender_distance_y,
            "rush_min_possession_retained_seconds": self.reducer.config().min_possession_retained_seconds,
        }))?))
    }
}

delegate_stats_reducer!(RushStatsModule, reducer);

impl Serialize for RushStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        StatsWithEventsExport {
            stats: self.reducer.stats(),
            events: self.reducer.events(),
        }
        .serialize(serializer)
    }
}

struct BallCarryStatsModule {
    reducer: BallCarryReducer,
}

impl StatsModule for BallCarryStatsModule {
    fn name(&self) -> &'static str {
        "ball_carry"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&TeamPlayerStatsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
        })?))
    }
}

delegate_stats_reducer!(BallCarryStatsModule, reducer);

impl Serialize for BallCarryStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TeamPlayerStatsWithEventsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
            events: self.reducer.carry_events(),
        }
        .serialize(serializer)
    }
}

struct FiftyFiftyStatsModule {
    reducer: FiftyFiftyReducer,
}

impl StatsModule for FiftyFiftyStatsModule {
    fn name(&self) -> &'static str {
        "fifty_fifty"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(
            &StatsWithPlayerStatsExport {
                stats: self.reducer.stats(),
                player_stats: player_stats_entries(self.reducer.player_stats()),
            },
        )?))
    }
}

delegate_stats_reducer!(FiftyFiftyStatsModule, reducer);

impl Serialize for FiftyFiftyStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        StatsWithPlayerEventsExport {
            stats: self.reducer.stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
            events: self.reducer.events(),
        }
        .serialize(serializer)
    }
}

struct CoreStatsModule {
    reducer: MatchStatsReducer,
}

impl StatsModule for CoreStatsModule {
    fn name(&self) -> &'static str {
        "core"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        let mut player_stats: Vec<_> = self
            .reducer
            .player_stats()
            .iter()
            .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                player_id: player_id.clone(),
                stats: CorePlayerStatsPlayback::from(stats),
            })
            .collect();
        player_stats.sort_by(|left, right| {
            format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
        });
        Ok(Some(serialize_to_json_value(&CoreStatsSnapshotExport {
            team_zero: self.reducer.team_zero_stats().into(),
            team_one: self.reducer.team_one_stats().into(),
            player_stats,
        })?))
    }
}

delegate_stats_reducer!(CoreStatsModule, reducer);

impl Serialize for CoreStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CoreStatsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
            timeline: self.reducer.timeline(),
        }
        .serialize(serializer)
    }
}

struct DemoStatsModule {
    reducer: DemoReducer,
}

impl StatsModule for DemoStatsModule {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn playback_frame_json(
        &self,
        _replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Option<serde_json::Value>> {
        Ok(Some(serialize_to_json_value(&TeamPlayerStatsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
        })?))
    }
}

delegate_stats_reducer!(DemoStatsModule, reducer);

impl Serialize for DemoStatsModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DemoStatsExport {
            team_zero: self.reducer.team_zero_stats(),
            team_one: self.reducer.team_one_stats(),
            player_stats: player_stats_entries(self.reducer.player_stats()),
            timeline: self.reducer.timeline(),
        }
        .serialize(serializer)
    }
}

struct BuiltinStatsModuleFactory {
    key: &'static str,
    name: &'static str,
    build: fn() -> Box<dyn StatsModule>,
}

impl StatsModuleFactory for BuiltinStatsModuleFactory {
    fn key(&self) -> String {
        self.key.to_owned()
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn build(&self) -> Box<dyn StatsModule> {
        (self.build)()
    }
}

fn builtin_factory(
    key: &'static str,
    build: fn() -> Box<dyn StatsModule>,
) -> Arc<dyn StatsModuleFactory> {
    Arc::new(BuiltinStatsModuleFactory {
        key,
        name: key,
        build,
    })
}

pub fn builtin_stats_module_names() -> &'static [&'static str] {
    &[
        "core",
        "backboard",
        "ceiling_shot",
        "double_tap",
        "fifty_fifty",
        "possession",
        "pressure",
        "rush",
        "touch",
        "speed_flip",
        "musty_flick",
        "dodge_reset",
        "ball_carry",
        "boost",
        "movement",
        "positioning",
        "powerslide",
        "demo",
    ]
}

pub fn builtin_stats_module_factory_by_name(name: &str) -> Option<Arc<dyn StatsModuleFactory>> {
    match name {
        "core" => Some(builtin_factory("core", || {
            Box::new(CoreStatsModule {
                reducer: MatchStatsReducer::default(),
            })
        })),
        "backboard" => Some(builtin_factory("backboard", || {
            Box::new(BackboardStatsModule {
                reducer: BackboardReducer::default(),
            })
        })),
        "ceiling_shot" => Some(builtin_factory("ceiling_shot", || {
            Box::new(CeilingShotStatsModule {
                reducer: CeilingShotReducer::default(),
            })
        })),
        "double_tap" => Some(builtin_factory("double_tap", || {
            Box::new(DoubleTapStatsModule {
                reducer: DoubleTapReducer::default(),
            })
        })),
        "fifty_fifty" => Some(builtin_factory("fifty_fifty", || {
            Box::new(FiftyFiftyStatsModule {
                reducer: FiftyFiftyReducer::default(),
            })
        })),
        "possession" => Some(builtin_factory("possession", || {
            Box::new(PossessionStatsModule {
                reducer: PossessionReducer::default(),
            })
        })),
        "pressure" => Some(builtin_factory("pressure", || {
            Box::new(PressureStatsModule {
                reducer: PressureReducer::default(),
            })
        })),
        "rush" => Some(builtin_factory("rush", || {
            Box::new(RushStatsModule {
                reducer: RushReducer::default(),
            })
        })),
        "touch" => Some(builtin_factory("touch", || {
            Box::new(TouchStatsModule {
                reducer: TouchReducer::default(),
            })
        })),
        "speed_flip" => Some(builtin_factory("speed_flip", || {
            Box::new(SpeedFlipStatsModule {
                reducer: SpeedFlipReducer::default(),
            })
        })),
        "musty_flick" => Some(builtin_factory("musty_flick", || {
            Box::new(MustyFlickStatsModule {
                reducer: MustyFlickReducer::default(),
            })
        })),
        "dodge_reset" => Some(builtin_factory("dodge_reset", || {
            Box::new(DodgeResetStatsModule {
                reducer: DodgeResetReducer::default(),
            })
        })),
        "ball_carry" => Some(builtin_factory("ball_carry", || {
            Box::new(BallCarryStatsModule {
                reducer: BallCarryReducer::default(),
            })
        })),
        "boost" => Some(builtin_factory("boost", || {
            Box::new(BoostStatsModule {
                reducer: BoostReducer::default(),
            })
        })),
        "movement" => Some(builtin_factory("movement", || {
            Box::new(MovementStatsModule {
                reducer: MovementReducer::default(),
            })
        })),
        "positioning" => Some(builtin_factory("positioning", || {
            Box::new(PositioningStatsModule {
                reducer: PositioningReducer::default(),
            })
        })),
        "powerslide" => Some(builtin_factory("powerslide", || {
            Box::new(PowerslideStatsModule {
                reducer: PowerslideReducer::default(),
            })
        })),
        "demo" => Some(builtin_factory("demo", || {
            Box::new(DemoStatsModule {
                reducer: DemoReducer::default(),
            })
        })),
        _ => None,
    }
}

pub fn builtin_stats_module_factories() -> Vec<Arc<dyn StatsModuleFactory>> {
    builtin_stats_module_names()
        .iter()
        .filter_map(|name| builtin_stats_module_factory_by_name(name))
        .collect()
}
