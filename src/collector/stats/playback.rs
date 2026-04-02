use boxcars::{Ps4Id, PsyNetId, RemoteId, SwitchId};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::*;

use super::types::serialize_to_json_value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsFrame<Modules> {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub modules: Modules,
}

pub type StatsPlaybackFrame = CapturedStatsFrame<Map<String, Value>>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsData<Frame> {
    pub replay_meta: ReplayMeta,
    pub config: Map<String, Value>,
    pub modules: Map<String, Value>,
    pub frames: Vec<Frame>,
}

pub type StatsPlaybackData = CapturedStatsData<StatsPlaybackFrame>;

impl<Modules> CapturedStatsFrame<Modules> {
    pub fn map_modules<Mapped, F>(
        self,
        transform: F,
    ) -> SubtrActorResult<CapturedStatsFrame<Mapped>>
    where
        F: FnOnce(Modules) -> SubtrActorResult<Mapped>,
    {
        Ok(CapturedStatsFrame {
            frame_number: self.frame_number,
            time: self.time,
            dt: self.dt,
            seconds_remaining: self.seconds_remaining,
            game_state: self.game_state,
            is_live_play: self.is_live_play,
            modules: transform(self.modules)?,
        })
    }
}

impl CapturedStatsData<StatsPlaybackFrame> {
    pub fn into_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_stats_timeline()
    }

    pub fn to_stats_timeline(&self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_replay_stats_timeline_with_frames(
            self.frames
                .iter()
                .map(|frame| self.replay_stats_frame(frame))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        )
    }

    pub(crate) fn into_replay_stats_timeline_with_frames(
        self,
        frames: Vec<ReplayStatsFrame>,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_replay_stats_timeline_with_frames(frames)
    }

    fn to_replay_stats_timeline_with_frames(
        &self,
        frames: Vec<ReplayStatsFrame>,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        Ok(ReplayStatsTimeline {
            config: self.legacy_config(),
            replay_meta: self.replay_meta.clone(),
            timeline_events: self.timeline_events_typed()?,
            backboard_events: self.module_player_events(
                "backboard",
                "events",
                parse_backboard_event,
            )?,
            ceiling_shot_events: self.module_player_events(
                "ceiling_shot",
                "events",
                parse_ceiling_shot_event,
            )?,
            double_tap_events: self.module_player_events(
                "double_tap",
                "events",
                parse_double_tap_event,
            )?,
            fifty_fifty_events: self.module_player_events(
                "fifty_fifty",
                "events",
                parse_fifty_fifty_event,
            )?,
            rush_events: self.module_typed_array("rush", "events")?,
            speed_flip_events: self.module_player_events(
                "speed_flip",
                "events",
                parse_speed_flip_event,
            )?,
            frames,
        })
    }

    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.to_stats_timeline_value()
    }

    pub fn to_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        let mut legacy = Map::new();
        legacy.insert("config".to_owned(), self.legacy_config_value()?);
        legacy.insert(
            "replay_meta".to_owned(),
            serialize_to_json_value(&self.replay_meta)?,
        );
        legacy.insert(
            "timeline_events".to_owned(),
            Value::Array(self.timeline_events()),
        );
        legacy.insert(
            "backboard_events".to_owned(),
            Value::Array(self.module_array("backboard", "events")),
        );
        legacy.insert(
            "ceiling_shot_events".to_owned(),
            Value::Array(self.module_array("ceiling_shot", "events")),
        );
        legacy.insert(
            "double_tap_events".to_owned(),
            Value::Array(self.module_array("double_tap", "events")),
        );
        legacy.insert(
            "fifty_fifty_events".to_owned(),
            Value::Array(self.module_array("fifty_fifty", "events")),
        );
        legacy.insert(
            "rush_events".to_owned(),
            Value::Array(self.module_array("rush", "events")),
        );
        legacy.insert(
            "speed_flip_events".to_owned(),
            Value::Array(self.module_array("speed_flip", "events")),
        );
        legacy.insert(
            "frames".to_owned(),
            Value::Array(
                self.frames
                    .iter()
                    .map(|frame| self.legacy_frame_value(frame))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(legacy))
    }

    pub fn into_legacy_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_stats_timeline_value()
    }

    pub fn to_legacy_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        self.to_stats_timeline_value()
    }

    fn timeline_events(&self) -> Vec<Value> {
        let mut events = self.module_array("core", "timeline");
        events.extend(self.module_array("demo", "timeline"));
        events.sort_by(|left, right| {
            let left_time = left.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            let right_time = right.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            left_time.total_cmp(&right_time)
        });
        events
    }

    fn timeline_events_typed(&self) -> SubtrActorResult<Vec<TimelineEvent>> {
        self.timeline_events()
            .iter()
            .map(parse_timeline_event)
            .collect()
    }

    fn legacy_config(&self) -> StatsTimelineConfig {
        let positioning_config = self.config.get("positioning").and_then(Value::as_object);
        let pressure_config = self.config.get("pressure").and_then(Value::as_object);
        let rush_config = self.config.get("rush").and_then(Value::as_object);
        let rush_defaults = RushReducerConfig::default();

        StatsTimelineConfig {
            most_back_forward_threshold_y: positioning_config
                .and_then(|config| config.get("most_back_forward_threshold_y"))
                .and_then(json_f32)
                .unwrap_or(PositioningReducerConfig::default().most_back_forward_threshold_y),
            pressure_neutral_zone_half_width_y: pressure_config
                .and_then(|config| config.get("pressure_neutral_zone_half_width_y"))
                .and_then(json_f32)
                .unwrap_or(PressureReducerConfig::default().neutral_zone_half_width_y),
            rush_max_start_y: rush_config
                .and_then(|config| config.get("rush_max_start_y"))
                .and_then(json_f32)
                .unwrap_or(rush_defaults.max_start_y),
            rush_attack_support_distance_y: rush_config
                .and_then(|config| config.get("rush_attack_support_distance_y"))
                .and_then(json_f32)
                .unwrap_or(rush_defaults.attack_support_distance_y),
            rush_defender_distance_y: rush_config
                .and_then(|config| config.get("rush_defender_distance_y"))
                .and_then(json_f32)
                .unwrap_or(rush_defaults.defender_distance_y),
            rush_min_possession_retained_seconds: rush_config
                .and_then(|config| config.get("rush_min_possession_retained_seconds"))
                .and_then(json_f32)
                .unwrap_or(rush_defaults.min_possession_retained_seconds),
        }
    }

    fn legacy_config_value(&self) -> SubtrActorResult<Value> {
        let positioning_config = self.config.get("positioning").and_then(Value::as_object);
        let pressure_config = self.config.get("pressure").and_then(Value::as_object);
        let rush_config = self.config.get("rush").and_then(Value::as_object);

        let mut config = Map::new();
        config.insert(
            "most_back_forward_threshold_y".to_owned(),
            serialize_to_json_value(
                &positioning_config
                    .and_then(|config| config.get("most_back_forward_threshold_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PositioningReducerConfig::default().most_back_forward_threshold_y as f64,
                    ),
            )?,
        );
        config.insert(
            "pressure_neutral_zone_half_width_y".to_owned(),
            serialize_to_json_value(
                &pressure_config
                    .and_then(|config| config.get("pressure_neutral_zone_half_width_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(PressureReducerConfig::default().neutral_zone_half_width_y as f64),
            )?,
        );
        let rush_defaults = RushReducerConfig::default();
        config.insert(
            "rush_max_start_y".to_owned(),
            serialize_to_json_value(
                &rush_config
                    .and_then(|config| config.get("rush_max_start_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(rush_defaults.max_start_y as f64),
            )?,
        );
        config.insert(
            "rush_attack_support_distance_y".to_owned(),
            serialize_to_json_value(
                &rush_config
                    .and_then(|config| config.get("rush_attack_support_distance_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(rush_defaults.attack_support_distance_y as f64),
            )?,
        );
        config.insert(
            "rush_defender_distance_y".to_owned(),
            serialize_to_json_value(
                &rush_config
                    .and_then(|config| config.get("rush_defender_distance_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(rush_defaults.defender_distance_y as f64),
            )?,
        );
        config.insert(
            "rush_min_possession_retained_seconds".to_owned(),
            serialize_to_json_value(
                &rush_config
                    .and_then(|config| config.get("rush_min_possession_retained_seconds"))
                    .and_then(Value::as_f64)
                    .unwrap_or(rush_defaults.min_possession_retained_seconds as f64),
            )?,
        );
        Ok(Value::Object(config))
    }

    fn legacy_frame_value(&self, frame: &StatsPlaybackFrame) -> SubtrActorResult<Value> {
        let mut legacy = Map::new();
        legacy.insert(
            "frame_number".to_owned(),
            serialize_to_json_value(&frame.frame_number)?,
        );
        legacy.insert("time".to_owned(), serialize_to_json_value(&frame.time)?);
        legacy.insert("dt".to_owned(), serialize_to_json_value(&frame.dt)?);
        legacy.insert(
            "seconds_remaining".to_owned(),
            serialize_to_json_value(&frame.seconds_remaining)?,
        );
        legacy.insert(
            "game_state".to_owned(),
            serialize_to_json_value(&frame.game_state)?,
        );
        legacy.insert(
            "is_live_play".to_owned(),
            serialize_to_json_value(&frame.is_live_play)?,
        );
        legacy.insert(
            "fifty_fifty".to_owned(),
            self.frame_stats_or_default::<FiftyFiftyStats>(frame, "fifty_fifty"),
        );
        legacy.insert(
            "possession".to_owned(),
            self.frame_stats_or_default::<PossessionStats>(frame, "possession"),
        );
        legacy.insert(
            "pressure".to_owned(),
            self.frame_stats_or_default::<PressureStats>(frame, "pressure"),
        );
        legacy.insert(
            "rush".to_owned(),
            self.frame_stats_or_default::<RushStats>(frame, "rush"),
        );
        legacy.insert(
            "team_zero".to_owned(),
            self.legacy_team_value(frame, "team_zero")?,
        );
        legacy.insert(
            "team_one".to_owned(),
            self.legacy_team_value(frame, "team_one")?,
        );
        legacy.insert(
            "players".to_owned(),
            Value::Array(
                self.replay_meta
                    .player_order()
                    .map(|player| self.legacy_player_value(frame, player))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(legacy))
    }

    pub(crate) fn replay_stats_frame(
        &self,
        frame: &StatsPlaybackFrame,
    ) -> SubtrActorResult<ReplayStatsFrame> {
        Ok(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: frame.game_state,
            is_live_play: frame.is_live_play,
            fifty_fifty: self.frame_stats_or_default_typed(frame, "fifty_fifty")?,
            possession: self.frame_stats_or_default_typed(frame, "possession")?,
            pressure: self.frame_stats_or_default_typed(frame, "pressure")?,
            rush: self.frame_stats_or_default_typed(frame, "rush")?,
            team_zero: self.replay_team_stats(frame, "team_zero")?,
            team_one: self.replay_team_stats(frame, "team_one")?,
            players: self
                .replay_meta
                .player_order()
                .map(|player| self.replay_player_stats(frame, player))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        })
    }

    fn replay_team_stats(
        &self,
        frame: &StatsPlaybackFrame,
        team_key: &str,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        Ok(TeamStatsSnapshot {
            core: self.frame_team_stat_or_default_typed(frame, "core", team_key)?,
            backboard: self.frame_team_stat_or_default_typed(frame, "backboard", team_key)?,
            double_tap: self.frame_team_stat_or_default_typed(frame, "double_tap", team_key)?,
            ball_carry: self.frame_team_stat_or_default_typed(frame, "ball_carry", team_key)?,
            boost: self.frame_team_stat_or_default_typed(frame, "boost", team_key)?,
            movement: self.frame_team_stat_or_default_typed(frame, "movement", team_key)?,
            powerslide: self.frame_team_stat_or_default_typed(frame, "powerslide", team_key)?,
            demo: self.frame_team_stat_or_default_typed(frame, "demo", team_key)?,
        })
    }

    fn replay_player_stats(
        &self,
        frame: &StatsPlaybackFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<PlayerStatsSnapshot> {
        let player_key = player_info_key(player)?;
        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: self.is_team_zero_player(player),
            core: self.frame_player_stat_or_default_typed_by_key(frame, "core", &player_key)?,
            backboard: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "backboard",
                &player_key,
            )?,
            ceiling_shot: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "ceiling_shot",
                &player_key,
            )?,
            double_tap: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "double_tap",
                &player_key,
            )?,
            fifty_fifty: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "fifty_fifty",
                &player_key,
            )?,
            speed_flip: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "speed_flip",
                &player_key,
            )?,
            touch: if frame.modules.contains_key("touch") {
                self.frame_player_stat_or_default_with_by_key(frame, "touch", &player_key, || {
                    TouchStats::default().with_complete_labeled_touch_counts()
                })?
            } else {
                self.frame_player_stat_or_default_typed_by_key(frame, "touch", &player_key)?
            },
            musty_flick: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "musty_flick",
                &player_key,
            )?,
            dodge_reset: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "dodge_reset",
                &player_key,
            )?,
            ball_carry: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "ball_carry",
                &player_key,
            )?,
            boost: self.frame_player_stat_or_default_typed_by_key(frame, "boost", &player_key)?,
            movement: self.frame_player_stat_or_default_with_by_key(
                frame,
                "movement",
                &player_key,
                || MovementStats::default().with_complete_labeled_tracked_time(),
            )?,
            positioning: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "positioning",
                &player_key,
            )?,
            powerslide: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "powerslide",
                &player_key,
            )?,
            demo: self.frame_player_stat_or_default_typed_by_key(frame, "demo", &player_key)?,
        })
    }

    fn is_team_zero_player(&self, player: &PlayerInfo) -> bool {
        self.replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    fn legacy_team_value(
        &self,
        frame: &StatsPlaybackFrame,
        team_key: &str,
    ) -> SubtrActorResult<Value> {
        let mut team = Map::new();
        team.insert(
            "core".to_owned(),
            self.frame_team_stat_or_default::<CoreTeamStats>(frame, "core", team_key),
        );
        team.insert(
            "backboard".to_owned(),
            self.frame_team_stat_or_default::<BackboardTeamStats>(frame, "backboard", team_key),
        );
        team.insert(
            "double_tap".to_owned(),
            self.frame_team_stat_or_default::<DoubleTapTeamStats>(frame, "double_tap", team_key),
        );
        team.insert(
            "ball_carry".to_owned(),
            self.frame_team_stat_or_default::<BallCarryStats>(frame, "ball_carry", team_key),
        );
        team.insert(
            "boost".to_owned(),
            self.frame_team_stat_or_default::<BoostStats>(frame, "boost", team_key),
        );
        team.insert(
            "movement".to_owned(),
            self.frame_team_stat_or_default::<MovementStats>(frame, "movement", team_key),
        );
        team.insert(
            "powerslide".to_owned(),
            self.frame_team_stat_or_default::<PowerslideStats>(frame, "powerslide", team_key),
        );
        team.insert(
            "demo".to_owned(),
            self.frame_team_stat_or_default::<DemoTeamStats>(frame, "demo", team_key),
        );
        Ok(Value::Object(team))
    }

    fn legacy_player_value(
        &self,
        frame: &StatsPlaybackFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<Value> {
        let player_key = player_info_key(player)?;
        let mut player_value = Map::new();
        player_value.insert(
            "player_id".to_owned(),
            serialize_to_json_value(&player.remote_id)?,
        );
        player_value.insert("name".to_owned(), serialize_to_json_value(&player.name)?);
        player_value.insert(
            "is_team_0".to_owned(),
            serialize_to_json_value(
                &self
                    .replay_meta
                    .team_zero
                    .iter()
                    .any(|team_player| team_player.remote_id == player.remote_id),
            )?,
        );
        player_value.insert(
            "core".to_owned(),
            self.frame_player_stat_or_default_by_key::<CorePlayerStats>(
                frame,
                "core",
                &player_key,
            )?,
        );
        player_value.insert(
            "backboard".to_owned(),
            self.frame_player_stat_or_default_by_key::<BackboardPlayerStats>(
                frame,
                "backboard",
                &player_key,
            )?,
        );
        player_value.insert(
            "ceiling_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<CeilingShotStats>(
                frame,
                "ceiling_shot",
                &player_key,
            )?,
        );
        player_value.insert(
            "double_tap".to_owned(),
            self.frame_player_stat_or_default_by_key::<DoubleTapPlayerStats>(
                frame,
                "double_tap",
                &player_key,
            )?,
        );
        player_value.insert(
            "fifty_fifty".to_owned(),
            self.frame_player_stat_or_default_by_key::<FiftyFiftyPlayerStats>(
                frame,
                "fifty_fifty",
                &player_key,
            )?,
        );
        player_value.insert(
            "speed_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<SpeedFlipStats>(
                frame,
                "speed_flip",
                &player_key,
            )?,
        );
        player_value.insert(
            "touch".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "touch",
                &player_key,
                if frame.modules.contains_key("touch") {
                    serialize_to_json_value(
                        &TouchStats::default().with_complete_labeled_touch_counts(),
                    )?
                } else {
                    default_json_value::<TouchStats>()
                },
            )?,
        );
        player_value.insert(
            "musty_flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<MustyFlickStats>(
                frame,
                "musty_flick",
                &player_key,
            )?,
        );
        player_value.insert(
            "dodge_reset".to_owned(),
            self.frame_player_stat_or_default_by_key::<DodgeResetStats>(
                frame,
                "dodge_reset",
                &player_key,
            )?,
        );
        player_value.insert(
            "ball_carry".to_owned(),
            self.frame_player_stat_or_default_by_key::<BallCarryStats>(
                frame,
                "ball_carry",
                &player_key,
            )?,
        );
        player_value.insert(
            "boost".to_owned(),
            self.frame_player_stat_or_default_by_key::<BoostStats>(frame, "boost", &player_key)?,
        );
        player_value.insert(
            "movement".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "movement",
                &player_key,
                if frame.modules.contains_key("movement") {
                    serialize_to_json_value(
                        &MovementStats::default().with_complete_labeled_tracked_time(),
                    )?
                } else {
                    default_json_value::<MovementStats>()
                },
            )?,
        );
        player_value.insert(
            "positioning".to_owned(),
            self.frame_player_stat_or_default_by_key::<PositioningStats>(
                frame,
                "positioning",
                &player_key,
            )?,
        );
        player_value.insert(
            "powerslide".to_owned(),
            self.frame_player_stat_or_default_by_key::<PowerslideStats>(
                frame,
                "powerslide",
                &player_key,
            )?,
        );
        player_value.insert(
            "demo".to_owned(),
            self.frame_player_stat_or_default_by_key::<DemoPlayerStats>(
                frame,
                "demo",
                &player_key,
            )?,
        );
        Ok(Value::Object(player_value))
    }

    fn frame_stats_or_default<T>(&self, frame: &StatsPlaybackFrame, module_name: &str) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get("stats"))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    fn frame_team_stat_or_default<T>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        team_key: &str,
    ) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(team_key))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    fn frame_player_stat_or_default_by_key<T>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<Value>
    where
        T: Default + Serialize,
    {
        self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            default_json_value::<T>(),
        )
    }

    fn frame_player_stat_or_value_by_key(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        player_key: &str,
        default_value: Value,
    ) -> SubtrActorResult<Value> {
        Ok(
            player_stats_value_for_key(frame.modules.get(module_name), player_key)?
                .cloned()
                .unwrap_or(default_value),
        )
    }

    fn frame_stats_or_default_typed<T>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_stats_or_default::<T>(frame, module_name))
    }

    fn frame_team_stat_or_default_typed<T>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        team_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_team_stat_or_default::<T>(frame, module_name, team_key))
    }

    fn frame_player_stat_or_default_typed_by_key<T>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        self.frame_player_stat_or_default_with_by_key(frame, module_name, player_key, T::default)
    }

    fn frame_player_stat_or_default_with_by_key<T, F>(
        &self,
        frame: &StatsPlaybackFrame,
        module_name: &str,
        player_key: &str,
        default: F,
    ) -> SubtrActorResult<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> T,
    {
        decode_json_value(self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            serialize_to_json_value(&default())?,
        )?)
    }

    fn module_typed_array<T>(&self, module_name: &str, field: &str) -> SubtrActorResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        decode_json_value(Value::Array(self.module_array(module_name, field)))
    }

    fn module_player_events<T, F>(
        &self,
        module_name: &str,
        field: &str,
        parse: F,
    ) -> SubtrActorResult<Vec<T>>
    where
        F: Fn(&Value) -> SubtrActorResult<T>,
    {
        self.module_array(module_name, field)
            .iter()
            .map(parse)
            .collect()
    }

    fn module_array(&self, module_name: &str, field: &str) -> Vec<Value> {
        self.modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(field))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }
}

impl CapturedStatsData<ReplayStatsFrame> {
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let CapturedStatsData {
            replay_meta,
            config,
            modules,
            frames,
        } = self;
        CapturedStatsData::<StatsPlaybackFrame> {
            replay_meta,
            config,
            modules,
            frames: Vec::new(),
        }
        .into_replay_stats_timeline_with_frames(frames)
    }
}

fn player_stats_value_for_key<'a>(
    module: Option<&'a Value>,
    player_key: &str,
) -> SubtrActorResult<Option<&'a Value>> {
    let Some(entries) = module
        .and_then(Value::as_object)
        .and_then(|module| module.get("player_stats"))
        .and_then(Value::as_array)
    else {
        return Ok(None);
    };

    for entry in entries {
        let Some(entry_object) = entry.as_object() else {
            continue;
        };
        let Some(player_id) = entry_object.get("player_id") else {
            continue;
        };
        let Some(player_stats) = entry_object.get("stats") else {
            continue;
        };
        if player_id_key(player_id)? == player_key {
            return Ok(Some(player_stats));
        }
    }

    Ok(None)
}

fn player_info_key(player: &PlayerInfo) -> SubtrActorResult<String> {
    player_id_key(&serialize_to_json_value(&player.remote_id)?)
}

fn player_id_key(player_id: &Value) -> SubtrActorResult<String> {
    serde_json::to_string(player_id).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

fn default_json_value<T>() -> Value
where
    T: Default + Serialize,
{
    serde_json::to_value(T::default()).expect("default stats should serialize to json")
}

fn decode_json_value<T>(value: Value) -> SubtrActorResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

fn parse_timeline_event(value: &Value) -> SubtrActorResult<TimelineEvent> {
    let object = json_object(value, "timeline event")?;
    Ok(TimelineEvent {
        time: json_required_f32(object, "time")?,
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        player_id: json_optional_remote_id(object.get("player_id"))?,
        is_team_0: json_optional_bool(object.get("is_team_0")),
    })
}

fn parse_backboard_event(value: &Value) -> SubtrActorResult<BackboardBounceEvent> {
    let object = json_object(value, "backboard event")?;
    Ok(BackboardBounceEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
    })
}

fn parse_ceiling_shot_event(value: &Value) -> SubtrActorResult<CeilingShotEvent> {
    let object = json_object(value, "ceiling shot event")?;
    Ok(CeilingShotEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        ceiling_contact_time: json_required_f32(object, "ceiling_contact_time")?,
        ceiling_contact_frame: json_required_usize(object, "ceiling_contact_frame")?,
        time_since_ceiling_contact: json_required_f32(object, "time_since_ceiling_contact")?,
        ceiling_contact_position: json_required_vec3(object, "ceiling_contact_position")?,
        touch_position: json_required_vec3(object, "touch_position")?,
        local_ball_position: json_required_vec3(object, "local_ball_position")?,
        separation_from_ceiling: json_required_f32(object, "separation_from_ceiling")?,
        roof_alignment: json_required_f32(object, "roof_alignment")?,
        forward_alignment: json_required_f32(object, "forward_alignment")?,
        forward_approach_speed: json_required_f32(object, "forward_approach_speed")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

fn parse_double_tap_event(value: &Value) -> SubtrActorResult<DoubleTapEvent> {
    let object = json_object(value, "double tap event")?;
    Ok(DoubleTapEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        backboard_time: json_required_f32(object, "backboard_time")?,
        backboard_frame: json_required_usize(object, "backboard_frame")?,
    })
}

fn parse_fifty_fifty_event(value: &Value) -> SubtrActorResult<FiftyFiftyEvent> {
    let object = json_object(value, "fifty fifty event")?;
    Ok(FiftyFiftyEvent {
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        resolve_time: json_required_f32(object, "resolve_time")?,
        resolve_frame: json_required_usize(object, "resolve_frame")?,
        is_kickoff: json_required_bool(object, "is_kickoff")?,
        team_zero_player: json_optional_remote_id(object.get("team_zero_player"))?,
        team_one_player: json_optional_remote_id(object.get("team_one_player"))?,
        team_zero_position: json_required_vec3(object, "team_zero_position")?,
        team_one_position: json_required_vec3(object, "team_one_position")?,
        midpoint: json_required_vec3(object, "midpoint")?,
        plane_normal: json_required_vec3(object, "plane_normal")?,
        winning_team_is_team_0: json_optional_bool(object.get("winning_team_is_team_0")),
        possession_team_is_team_0: json_optional_bool(object.get("possession_team_is_team_0")),
    })
}

fn parse_speed_flip_event(value: &Value) -> SubtrActorResult<SpeedFlipEvent> {
    let object = json_object(value, "speed flip event")?;
    Ok(SpeedFlipEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        time_since_kickoff_start: json_required_f32(object, "time_since_kickoff_start")?,
        start_position: json_required_vec3(object, "start_position")?,
        end_position: json_required_vec3(object, "end_position")?,
        start_speed: json_required_f32(object, "start_speed")?,
        max_speed: json_required_f32(object, "max_speed")?,
        best_alignment: json_required_f32(object, "best_alignment")?,
        diagonal_score: json_required_f32(object, "diagonal_score")?,
        cancel_score: json_required_f32(object, "cancel_score")?,
        speed_score: json_required_f32(object, "speed_score")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

fn json_object<'a>(
    value: &'a Value,
    context: &str,
) -> SubtrActorResult<&'a serde_json::Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Expected {context} to be a JSON object"
        )))
    })
}

fn json_required_value<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<&'a Value> {
    object.get(field).ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Missing JSON field '{field}'"
        )))
    })
}

fn json_f32(value: &Value) -> Option<f32> {
    value.as_f64().map(|number| number as f32)
}

fn json_required_f32(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<f32> {
    json_f32(json_required_value(object, field)?).ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Expected JSON field '{field}' to be a float"
        )))
    })
}

fn json_required_usize(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<usize> {
    json_required_value(object, field)?
        .as_u64()
        .map(|number| number as usize)
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be an unsigned integer"
            )))
        })
}

fn json_required_bool(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<bool> {
    json_required_value(object, field)?
        .as_bool()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a bool"
            )))
        })
}

fn json_optional_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn json_required_vec3(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<[f32; 3]> {
    let array = json_required_value(object, field)?
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a 3-element array"
            )))
        })?;
    if array.len() != 3 {
        return SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            format!("Expected JSON field '{field}' to contain exactly 3 elements"),
        ));
    }
    Ok([
        json_f32(&array[0]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[0]' to be a float"
            )))
        })?,
        json_f32(&array[1]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[1]' to be a float"
            )))
        })?,
        json_f32(&array[2]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[2]' to be a float"
            )))
        })?,
    ])
}

fn json_required_remote_id(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<PlayerId> {
    json_remote_id(json_required_value(object, field)?)
}

fn json_optional_remote_id(value: Option<&Value>) -> SubtrActorResult<Option<PlayerId>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(json_remote_id(value)?)),
    }
}

fn json_remote_id(value: &Value) -> SubtrActorResult<PlayerId> {
    let object = json_object(value, "remote id")?;
    if object.len() != 1 {
        return SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            "Expected remote id to contain exactly one variant".to_owned(),
        ));
    }

    let (variant, payload) = object.iter().next().expect("validated single variant");
    match variant.as_str() {
        "PlayStation" => {
            let payload = json_object(payload, "playstation remote id")?;
            Ok(RemoteId::PlayStation(Ps4Id {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                name: json_required_value(payload, "name")?
                    .as_str()
                    .ok_or_else(|| {
                        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                            "Expected PlayStation name to be a string".to_owned(),
                        ))
                    })?
                    .to_owned(),
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "PsyNet" => {
            let payload = json_object(payload, "psynet remote id")?;
            Ok(RemoteId::PsyNet(PsyNetId {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "SplitScreen" => Ok(RemoteId::SplitScreen(json_u64(payload)? as u32)),
        "Steam" => Ok(RemoteId::Steam(json_u64(payload)?)),
        "Switch" => {
            let payload = json_object(payload, "switch remote id")?;
            Ok(RemoteId::Switch(SwitchId {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "Xbox" => Ok(RemoteId::Xbox(json_u64(payload)?)),
        "QQ" => Ok(RemoteId::QQ(json_u64(payload)?)),
        "Epic" => Ok(RemoteId::Epic(
            payload
                .as_str()
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected Epic remote id payload to be a string".to_owned(),
                    ))
                })?
                .to_owned(),
        )),
        variant => SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            format!("Unknown remote id variant '{variant}'"),
        )),
    }
}

fn json_u64(value: &Value) -> SubtrActorResult<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                "Expected JSON value to be a u64".to_owned(),
            ))
        })
}

fn json_u8_vec(value: &Value) -> SubtrActorResult<Vec<u8>> {
    value
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                "Expected JSON value to be an array of bytes".to_owned(),
            ))
        })?
        .iter()
        .map(|entry| {
            entry
                .as_u64()
                .and_then(|number| u8::try_from(number).ok())
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected JSON array entry to be a byte".to_owned(),
                    ))
                })
        })
        .collect()
}
