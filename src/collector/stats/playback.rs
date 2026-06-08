use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::*;

use super::types::serialize_to_json_value;

#[path = "playback_event_parsers.rs"]
mod playback_event_parsers;
#[path = "playback_events.rs"]
mod playback_events;
#[path = "playback_frames.rs"]
mod playback_frames;
#[path = "playback_json.rs"]
mod playback_json;
#[path = "playback_mechanics.rs"]
mod playback_mechanics;
use playback_event_parsers::*;
use playback_json::*;
use playback_mechanics::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsFrame<Modules> {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    pub modules: Modules,
}

pub type StatsSnapshotFrame = CapturedStatsFrame<Map<String, Value>>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsData<Frame> {
    pub replay_meta: ReplayMeta,
    pub config: Map<String, Value>,
    pub modules: Map<String, Value>,
    pub frames: Vec<Frame>,
}

pub type StatsSnapshotData = CapturedStatsData<StatsSnapshotFrame>;

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
            ball_has_been_hit: self.ball_has_been_hit,
            kickoff_countdown_time: self.kickoff_countdown_time,
            gameplay_phase: self.gameplay_phase,
            is_live_play: self.is_live_play,
            modules: transform(self.modules)?,
        })
    }
}

impl CapturedStatsData<StatsSnapshotFrame> {
    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }

    pub fn into_legacy_replay_stats_timeline_with_progress<F>(
        self,
        frame_interval: usize,
        mut on_progress: F,
    ) -> SubtrActorResult<ReplayStatsTimeline>
    where
        F: FnMut(usize, usize) -> SubtrActorResult<()>,
    {
        let frame_interval = frame_interval.max(1);
        let total_frames = self.frames.len();
        on_progress(0, total_frames)?;
        let frames = self
            .frames
            .iter()
            .enumerate()
            .map(|(frame_index, frame)| {
                let replay_frame = self.replay_stats_frame(frame)?;
                let processed_frames = frame_index + 1;
                if processed_frames == total_frames
                    || processed_frames.is_multiple_of(frame_interval)
                {
                    on_progress(processed_frames, total_frames)?;
                }
                Ok(replay_frame)
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        self.to_replay_stats_timeline_with_frames(frames)
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline_with_progress for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_with_progress<F>(
        self,
        frame_interval: usize,
        on_progress: F,
    ) -> SubtrActorResult<ReplayStatsTimeline>
    where
        F: FnMut(usize, usize) -> SubtrActorResult<()>,
    {
        self.into_legacy_replay_stats_timeline_with_progress(frame_interval, on_progress)
    }

    pub fn to_legacy_replay_stats_timeline(&self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_replay_stats_timeline_with_frames(
            self.frames
                .iter()
                .map(|frame| self.replay_stats_frame(frame))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        )
    }

    #[deprecated(
        note = "use to_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn to_stats_timeline(&self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_legacy_replay_stats_timeline()
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
            config: self.timeline_config(),
            replay_meta: self.replay_meta.clone(),
            events: self.timeline_event_sets_typed()?,
            frames,
        })
    }

    pub fn into_legacy_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.to_legacy_stats_timeline_value()
    }

    #[deprecated(
        note = "use into_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_legacy_stats_timeline_value()
    }

    pub fn to_legacy_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        let mut timeline = Map::new();
        timeline.insert("config".to_owned(), self.timeline_config_value()?);
        timeline.insert(
            "replay_meta".to_owned(),
            serialize_to_json_value(&self.replay_meta)?,
        );
        timeline.insert("events".to_owned(), self.timeline_event_sets_value()?);
        timeline.insert(
            "frames".to_owned(),
            Value::Array(
                self.frames
                    .iter()
                    .map(|frame| self.timeline_frame_value(frame))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(timeline))
    }

    #[deprecated(
        note = "use to_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn to_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        self.to_legacy_stats_timeline_value()
    }

    fn timeline_config(&self) -> StatsTimelineConfig {
        let positioning_config = self.config.get("positioning").and_then(Value::as_object);
        let pressure_config = self.config.get("pressure").and_then(Value::as_object);
        let territorial_pressure_config = self
            .config
            .get("territorial_pressure")
            .and_then(Value::as_object);
        let territorial_pressure_defaults = TerritorialPressureCalculatorConfig::default();
        let rotation_config = self.config.get("rotation").and_then(Value::as_object);
        let rotation_defaults = RotationCalculatorConfig::default();
        let rush_config = self.config.get("rush").and_then(Value::as_object);
        let rush_defaults = RushCalculatorConfig::default();
        let aerial_goal_config = self.config.get("aerial_goal").and_then(Value::as_object);
        let high_aerial_goal_config = self
            .config
            .get("high_aerial_goal")
            .and_then(Value::as_object);
        let long_distance_goal_config = self
            .config
            .get("long_distance_goal")
            .and_then(Value::as_object);
        let own_half_goal_config = self.config.get("own_half_goal").and_then(Value::as_object);
        let empty_net_goal_config = self.config.get("empty_net_goal").and_then(Value::as_object);
        let flick_goal_config = self.config.get("flick_goal").and_then(Value::as_object);
        let double_tap_goal_config = self
            .config
            .get("double_tap_goal")
            .and_then(Value::as_object);
        let one_timer_goal_config = self.config.get("one_timer_goal").and_then(Value::as_object);
        let air_dribble_goal_config = self
            .config
            .get("air_dribble_goal")
            .and_then(Value::as_object);
        let flip_reset_goal_config = self
            .config
            .get("flip_reset_goal")
            .and_then(Value::as_object);
        let bump_goal_config = self.config.get("bump_goal").and_then(Value::as_object);
        let demo_goal_config = self.config.get("demo_goal").and_then(Value::as_object);
        let half_volley_config = self.config.get("half_volley").and_then(Value::as_object);
        let half_volley_goal_config = self
            .config
            .get("half_volley_goal")
            .and_then(Value::as_object);

        StatsTimelineConfig {
            most_back_forward_threshold_y: positioning_config
                .and_then(|config| config.get("most_back_forward_threshold_y"))
                .and_then(json_f32)
                .unwrap_or(PositioningCalculatorConfig::default().most_back_forward_threshold_y),
            level_ball_depth_margin: positioning_config
                .and_then(|config| config.get("level_ball_depth_margin"))
                .and_then(json_f32)
                .unwrap_or(PositioningCalculatorConfig::default().level_ball_depth_margin),
            closest_to_ball_switch_margin: positioning_config
                .and_then(|config| config.get("closest_to_ball_switch_margin"))
                .and_then(json_f32)
                .unwrap_or(PositioningCalculatorConfig::default().closest_to_ball_switch_margin),
            closest_to_ball_switch_min_seconds: positioning_config
                .and_then(|config| config.get("closest_to_ball_switch_min_seconds"))
                .and_then(json_f32)
                .unwrap_or(
                    PositioningCalculatorConfig::default().closest_to_ball_switch_min_seconds,
                ),
            pressure_neutral_zone_half_width_y: pressure_config
                .and_then(|config| config.get("pressure_neutral_zone_half_width_y"))
                .and_then(json_f32)
                .unwrap_or(PressureCalculatorConfig::default().neutral_zone_half_width_y),
            territorial_pressure_neutral_zone_half_width_y: territorial_pressure_config
                .and_then(|config| config.get("territorial_pressure_neutral_zone_half_width_y"))
                .and_then(json_f32)
                .unwrap_or(territorial_pressure_defaults.neutral_zone_half_width_y),
            territorial_pressure_min_establish_seconds: territorial_pressure_config
                .and_then(|config| config.get("territorial_pressure_min_establish_seconds"))
                .and_then(json_f32)
                .unwrap_or(territorial_pressure_defaults.min_establish_seconds),
            territorial_pressure_min_establish_third_seconds: territorial_pressure_config
                .and_then(|config| config.get("territorial_pressure_min_establish_third_seconds"))
                .and_then(json_f32)
                .unwrap_or(territorial_pressure_defaults.min_establish_third_seconds),
            territorial_pressure_relief_grace_seconds: territorial_pressure_config
                .and_then(|config| config.get("territorial_pressure_relief_grace_seconds"))
                .and_then(json_f32)
                .unwrap_or(territorial_pressure_defaults.relief_grace_seconds),
            territorial_pressure_confirmed_relief_grace_seconds: territorial_pressure_config
                .and_then(|config| {
                    config.get("territorial_pressure_confirmed_relief_grace_seconds")
                })
                .and_then(json_f32)
                .unwrap_or(territorial_pressure_defaults.confirmed_relief_grace_seconds),
            rotation_role_depth_margin: rotation_config
                .and_then(|config| config.get("role_depth_margin"))
                .and_then(json_f32)
                .unwrap_or(rotation_defaults.role_depth_margin),
            rotation_first_man_ambiguity_margin: rotation_config
                .and_then(|config| config.get("first_man_ambiguity_margin"))
                .and_then(json_f32)
                .unwrap_or(rotation_defaults.first_man_ambiguity_margin),
            rotation_first_man_debounce_seconds: rotation_config
                .and_then(|config| config.get("first_man_debounce_seconds"))
                .and_then(json_f32)
                .unwrap_or(rotation_defaults.first_man_debounce_seconds),
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
            aerial_goal_min_ball_z: aerial_goal_config
                .and_then(|config| config.get("aerial_goal_min_ball_z"))
                .and_then(json_f32)
                .unwrap_or(AerialGoalCalculatorConfig::default().min_ball_z),
            high_aerial_goal_min_ball_z: high_aerial_goal_config
                .and_then(|config| config.get("high_aerial_goal_min_ball_z"))
                .and_then(json_f32)
                .unwrap_or(HighAerialGoalCalculatorConfig::default().min_ball_z),
            long_distance_goal_max_attacking_y: long_distance_goal_config
                .and_then(|config| config.get("long_distance_goal_max_attacking_y"))
                .and_then(json_f32)
                .unwrap_or(LongDistanceGoalCalculatorConfig::default().max_attacking_y),
            own_half_goal_max_attacking_y: own_half_goal_config
                .and_then(|config| config.get("own_half_goal_max_attacking_y"))
                .and_then(json_f32)
                .unwrap_or(OwnHalfGoalCalculatorConfig::default().max_attacking_y),
            empty_net_min_defender_y_margin: empty_net_goal_config
                .and_then(|config| config.get("empty_net_min_defender_y_margin"))
                .and_then(json_f32)
                .unwrap_or(EmptyNetGoalCalculatorConfig::default().min_defender_y_margin),
            empty_net_min_defender_distance: empty_net_goal_config
                .and_then(|config| config.get("empty_net_min_defender_distance"))
                .and_then(json_f32)
                .unwrap_or(EmptyNetGoalCalculatorConfig::default().min_defender_distance),
            empty_net_max_touch_attacking_y: empty_net_goal_config
                .and_then(|config| config.get("empty_net_max_touch_attacking_y"))
                .and_then(json_f32)
                .unwrap_or(EmptyNetGoalCalculatorConfig::default().max_touch_attacking_y),
            flick_goal_max_event_to_goal_seconds: json_config_f32(
                flick_goal_config,
                "flick_goal_max_event_to_goal_seconds",
                "flick_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(FlickGoalCalculatorConfig::default().max_event_to_goal_seconds),
            double_tap_goal_max_event_to_goal_seconds: json_config_f32(
                double_tap_goal_config,
                "double_tap_goal_max_event_to_goal_seconds",
                "double_tap_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(DoubleTapGoalCalculatorConfig::default().max_event_to_goal_seconds),
            one_timer_goal_max_event_to_goal_seconds: json_config_f32(
                one_timer_goal_config,
                "one_timer_goal_max_event_to_goal_seconds",
                "one_timer_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(OneTimerGoalCalculatorConfig::default().max_event_to_goal_seconds),
            air_dribble_goal_max_end_to_goal_seconds: json_config_f32(
                air_dribble_goal_config,
                "air_dribble_goal_max_end_to_goal_seconds",
                "air_dribble_goal_max_end_to_touch_seconds",
            )
            .unwrap_or(AirDribbleGoalCalculatorConfig::default().max_end_to_goal_seconds),
            flip_reset_goal_max_event_to_goal_seconds: json_config_f32(
                flip_reset_goal_config,
                "flip_reset_goal_max_event_to_goal_seconds",
                "flip_reset_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(FlipResetGoalCalculatorConfig::default().max_event_to_goal_seconds),
            bump_goal_max_event_to_goal_seconds: json_config_f32(
                bump_goal_config,
                "bump_goal_max_event_to_goal_seconds",
                "bump_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(BumpGoalCalculatorConfig::default().max_event_to_goal_seconds),
            demo_goal_max_event_to_goal_seconds: json_config_f32(
                demo_goal_config,
                "demo_goal_max_event_to_goal_seconds",
                "demo_goal_max_event_to_touch_seconds",
            )
            .unwrap_or(DemoGoalCalculatorConfig::default().max_event_to_goal_seconds),
            half_volley_max_bounce_to_touch_seconds: half_volley_config
                .and_then(|config| config.get("half_volley_max_bounce_to_touch_seconds"))
                .and_then(json_f32)
                .unwrap_or(HalfVolleyCalculatorConfig::default().max_bounce_to_touch_seconds),
            half_volley_min_ball_speed: half_volley_config
                .and_then(|config| config.get("half_volley_min_ball_speed"))
                .and_then(json_f32)
                .unwrap_or(HalfVolleyCalculatorConfig::default().min_ball_speed),
            half_volley_goal_max_touch_to_goal_seconds: half_volley_goal_config
                .and_then(|config| config.get("half_volley_goal_max_touch_to_goal_seconds"))
                .and_then(json_f32)
                .unwrap_or(HalfVolleyGoalCalculatorConfig::default().max_touch_to_goal_seconds),
            half_volley_goal_min_goal_alignment: half_volley_goal_config
                .and_then(|config| config.get("half_volley_goal_min_goal_alignment"))
                .and_then(json_f32)
                .unwrap_or(HalfVolleyGoalCalculatorConfig::default().min_goal_alignment),
        }
    }

    fn timeline_config_value(&self) -> SubtrActorResult<Value> {
        let positioning_config = self.config.get("positioning").and_then(Value::as_object);
        let pressure_config = self.config.get("pressure").and_then(Value::as_object);
        let territorial_pressure_config = self
            .config
            .get("territorial_pressure")
            .and_then(Value::as_object);
        let rotation_config = self.config.get("rotation").and_then(Value::as_object);
        let rush_config = self.config.get("rush").and_then(Value::as_object);
        let aerial_goal_config = self.config.get("aerial_goal").and_then(Value::as_object);
        let high_aerial_goal_config = self
            .config
            .get("high_aerial_goal")
            .and_then(Value::as_object);
        let long_distance_goal_config = self
            .config
            .get("long_distance_goal")
            .and_then(Value::as_object);
        let own_half_goal_config = self.config.get("own_half_goal").and_then(Value::as_object);
        let empty_net_goal_config = self.config.get("empty_net_goal").and_then(Value::as_object);
        let flick_goal_config = self.config.get("flick_goal").and_then(Value::as_object);
        let double_tap_goal_config = self
            .config
            .get("double_tap_goal")
            .and_then(Value::as_object);
        let one_timer_goal_config = self.config.get("one_timer_goal").and_then(Value::as_object);
        let air_dribble_goal_config = self
            .config
            .get("air_dribble_goal")
            .and_then(Value::as_object);
        let flip_reset_goal_config = self
            .config
            .get("flip_reset_goal")
            .and_then(Value::as_object);
        let bump_goal_config = self.config.get("bump_goal").and_then(Value::as_object);
        let demo_goal_config = self.config.get("demo_goal").and_then(Value::as_object);
        let half_volley_config = self.config.get("half_volley").and_then(Value::as_object);
        let half_volley_goal_config = self
            .config
            .get("half_volley_goal")
            .and_then(Value::as_object);

        let mut config = Map::new();
        config.insert(
            "most_back_forward_threshold_y".to_owned(),
            serialize_to_json_value(
                &positioning_config
                    .and_then(|config| config.get("most_back_forward_threshold_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PositioningCalculatorConfig::default().most_back_forward_threshold_y as f64,
                    ),
            )?,
        );
        config.insert(
            "level_ball_depth_margin".to_owned(),
            serialize_to_json_value(
                &positioning_config
                    .and_then(|config| config.get("level_ball_depth_margin"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PositioningCalculatorConfig::default().level_ball_depth_margin as f64,
                    ),
            )?,
        );
        config.insert(
            "closest_to_ball_switch_margin".to_owned(),
            serialize_to_json_value(
                &positioning_config
                    .and_then(|config| config.get("closest_to_ball_switch_margin"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PositioningCalculatorConfig::default().closest_to_ball_switch_margin as f64,
                    ),
            )?,
        );
        config.insert(
            "closest_to_ball_switch_min_seconds".to_owned(),
            serialize_to_json_value(
                &positioning_config
                    .and_then(|config| config.get("closest_to_ball_switch_min_seconds"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PositioningCalculatorConfig::default().closest_to_ball_switch_min_seconds
                            as f64,
                    ),
            )?,
        );
        config.insert(
            "pressure_neutral_zone_half_width_y".to_owned(),
            serialize_to_json_value(
                &pressure_config
                    .and_then(|config| config.get("pressure_neutral_zone_half_width_y"))
                    .and_then(Value::as_f64)
                    .unwrap_or(
                        PressureCalculatorConfig::default().neutral_zone_half_width_y as f64,
                    ),
            )?,
        );
        let territorial_pressure_defaults = TerritorialPressureCalculatorConfig::default();
        for (key, default_value) in [
            (
                "territorial_pressure_neutral_zone_half_width_y",
                territorial_pressure_defaults.neutral_zone_half_width_y,
            ),
            (
                "territorial_pressure_min_establish_seconds",
                territorial_pressure_defaults.min_establish_seconds,
            ),
            (
                "territorial_pressure_min_establish_third_seconds",
                territorial_pressure_defaults.min_establish_third_seconds,
            ),
            (
                "territorial_pressure_relief_grace_seconds",
                territorial_pressure_defaults.relief_grace_seconds,
            ),
            (
                "territorial_pressure_confirmed_relief_grace_seconds",
                territorial_pressure_defaults.confirmed_relief_grace_seconds,
            ),
        ] {
            config.insert(
                key.to_owned(),
                serialize_to_json_value(
                    &territorial_pressure_config
                        .and_then(|config| config.get(key))
                        .and_then(Value::as_f64)
                        .unwrap_or(default_value as f64),
                )?,
            );
        }
        let rotation_defaults = RotationCalculatorConfig::default();
        for (key, default_value) in [
            (
                "rotation_role_depth_margin",
                rotation_defaults.role_depth_margin,
            ),
            (
                "rotation_first_man_ambiguity_margin",
                rotation_defaults.first_man_ambiguity_margin,
            ),
            (
                "rotation_first_man_debounce_seconds",
                rotation_defaults.first_man_debounce_seconds,
            ),
        ] {
            let source_key = key.strip_prefix("rotation_").unwrap_or(key);
            config.insert(
                key.to_owned(),
                serialize_to_json_value(
                    &rotation_config
                        .and_then(|config| config.get(source_key))
                        .and_then(Value::as_f64)
                        .unwrap_or(default_value as f64),
                )?,
            );
        }
        let rush_defaults = RushCalculatorConfig::default();
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
        for (module_config, key, default_value) in [
            (
                aerial_goal_config,
                "aerial_goal_min_ball_z",
                AerialGoalCalculatorConfig::default().min_ball_z,
            ),
            (
                high_aerial_goal_config,
                "high_aerial_goal_min_ball_z",
                HighAerialGoalCalculatorConfig::default().min_ball_z,
            ),
            (
                long_distance_goal_config,
                "long_distance_goal_max_attacking_y",
                LongDistanceGoalCalculatorConfig::default().max_attacking_y,
            ),
            (
                own_half_goal_config,
                "own_half_goal_max_attacking_y",
                OwnHalfGoalCalculatorConfig::default().max_attacking_y,
            ),
            (
                empty_net_goal_config,
                "empty_net_min_defender_y_margin",
                EmptyNetGoalCalculatorConfig::default().min_defender_y_margin,
            ),
            (
                empty_net_goal_config,
                "empty_net_min_defender_distance",
                EmptyNetGoalCalculatorConfig::default().min_defender_distance,
            ),
            (
                empty_net_goal_config,
                "empty_net_max_touch_attacking_y",
                EmptyNetGoalCalculatorConfig::default().max_touch_attacking_y,
            ),
            (
                flick_goal_config,
                "flick_goal_max_event_to_goal_seconds",
                FlickGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                double_tap_goal_config,
                "double_tap_goal_max_event_to_goal_seconds",
                DoubleTapGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                one_timer_goal_config,
                "one_timer_goal_max_event_to_goal_seconds",
                OneTimerGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                air_dribble_goal_config,
                "air_dribble_goal_max_end_to_goal_seconds",
                AirDribbleGoalCalculatorConfig::default().max_end_to_goal_seconds,
            ),
            (
                flip_reset_goal_config,
                "flip_reset_goal_max_event_to_goal_seconds",
                FlipResetGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                bump_goal_config,
                "bump_goal_max_event_to_goal_seconds",
                BumpGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                demo_goal_config,
                "demo_goal_max_event_to_goal_seconds",
                DemoGoalCalculatorConfig::default().max_event_to_goal_seconds,
            ),
            (
                half_volley_config,
                "half_volley_max_bounce_to_touch_seconds",
                HalfVolleyCalculatorConfig::default().max_bounce_to_touch_seconds,
            ),
            (
                half_volley_config,
                "half_volley_min_ball_speed",
                HalfVolleyCalculatorConfig::default().min_ball_speed,
            ),
            (
                half_volley_goal_config,
                "half_volley_goal_max_touch_to_goal_seconds",
                HalfVolleyGoalCalculatorConfig::default().max_touch_to_goal_seconds,
            ),
            (
                half_volley_goal_config,
                "half_volley_goal_min_goal_alignment",
                HalfVolleyGoalCalculatorConfig::default().min_goal_alignment,
            ),
        ] {
            config.insert(
                key.to_owned(),
                serialize_to_json_value(
                    &module_config
                        .and_then(|config| config.get(key))
                        .and_then(Value::as_f64)
                        .unwrap_or(default_value as f64),
                )?,
            );
        }
        Ok(Value::Object(config))
    }
}

impl CapturedStatsData<ReplayStatsFrame> {
    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let CapturedStatsData {
            replay_meta,
            config,
            modules,
            frames,
        } = self;
        CapturedStatsData::<StatsSnapshotFrame> {
            replay_meta,
            config,
            modules,
            frames: Vec::new(),
        }
        .into_replay_stats_timeline_with_frames(frames)
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }
}
