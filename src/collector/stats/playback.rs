use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::*;

use super::types::serialize_to_json_value;

#[path = "playback_event_parsers.rs"]
mod playback_event_parsers;
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

    fn goal_tag_events_typed(&self) -> SubtrActorResult<Vec<GoalTagEvent>> {
        let mut events = Vec::new();
        for module_name in [
            "aerial_goal",
            "high_aerial_goal",
            "long_distance_goal",
            "own_half_goal",
            "empty_net_goal",
            "counter_attack_goal",
            "flick_goal",
            "double_tap_goal",
            "one_timer_goal",
            "passing_goal",
            "air_dribble_goal",
            "flip_reset_goal",
            "half_volley_goal",
        ] {
            events.extend(self.module_player_events(
                module_name,
                "events",
                parse_goal_tag_event,
            )?);
        }
        events.sort_by(|left, right| {
            left.time
                .total_cmp(&right.time)
                .then_with(|| left.frame.cmp(&right.frame))
                .then_with(|| left.goal_index.cmp(&right.goal_index))
                .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
        });
        Ok(events)
    }

    fn mechanic_events_typed(&self) -> SubtrActorResult<Vec<StatsTimelineTagEvent>> {
        let mut events = Vec::new();

        for (index, value) in self.module_array("ball_carry", "events").iter().enumerate() {
            events.push(parse_ball_carry_mechanic_event(value, index)?);
        }
        for (index, value) in self
            .module_array("ceiling_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_ceiling_shot_event(value)?;
            events.push(span_mechanic_event(
                "ceiling_shot",
                index,
                event.ceiling_contact_frame,
                event.frame,
                event.ceiling_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("wall_aerial", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        for (index, value) in self
            .module_array("wall_aerial_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_shot_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial_shot",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        for (index, value) in self.module_array("center", "events").iter().enumerate() {
            let event = parse_center_event(value)?;
            events.push(span_mechanic_event(
                "center",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("dodge_reset", "on_ball_events")
            .iter()
            .enumerate()
        {
            events.push(parse_dodge_reset_mechanic_event(value, index)?);
        }
        for (index, value) in self.module_array("double_tap", "events").iter().enumerate() {
            let event = parse_double_tap_event(value)?;
            events.push(span_mechanic_event(
                "double_tap",
                index,
                event.backboard_frame,
                event.frame,
                event.backboard_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("flick", "events").iter().enumerate() {
            events.push(parse_flick_mechanic_event(value, index)?);
        }
        for (index, value) in self
            .module_array("musty_flick", "events")
            .iter()
            .enumerate()
        {
            events.push(parse_musty_flick_mechanic_event(value, index)?);
        }
        for (index, value) in self.module_array("one_timer", "events").iter().enumerate() {
            let event = parse_one_timer_event(value)?;
            events.push(span_mechanic_event(
                "one_timer",
                index,
                event.pass_start_frame,
                event.frame,
                event.pass_start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("pass", "events").iter().enumerate() {
            let event = parse_pass_event(value)?;
            events.push(span_mechanic_event(
                "pass",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.passer,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("speed_flip", "events").iter().enumerate() {
            let event = parse_speed_flip_event(value)?;
            events.push(moment_mechanic_event(
                "speed_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("half_flip", "events").iter().enumerate() {
            let event = parse_half_flip_event(value)?;
            events.push(moment_mechanic_event(
                "half_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("half_volley", "events")
            .iter()
            .enumerate()
        {
            let event = parse_half_volley_event(value)?;
            events.push(moment_mechanic_event(
                "half_volley",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("wavedash", "events").iter().enumerate() {
            let event = parse_wavedash_event(value)?;
            events.push(span_mechanic_event(
                "wavedash",
                index,
                event.dodge_frame,
                event.frame,
                event.dodge_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        events.sort_by(|left, right| {
            let left_time = mechanic_event_start_time(left);
            let right_time = mechanic_event_start_time(right);
            left_time
                .total_cmp(&right_time)
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(events)
    }

    fn goal_tag_events_value(&self) -> Vec<Value> {
        let mut events = Vec::new();
        for module_name in [
            "aerial_goal",
            "high_aerial_goal",
            "long_distance_goal",
            "own_half_goal",
            "empty_net_goal",
            "counter_attack_goal",
            "flick_goal",
            "double_tap_goal",
            "one_timer_goal",
            "passing_goal",
            "air_dribble_goal",
            "flip_reset_goal",
            "half_volley_goal",
        ] {
            events.extend(self.module_array(module_name, "events"));
        }
        events.sort_by(|left, right| {
            let left_time = left.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            let right_time = right.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            left_time.total_cmp(&right_time)
        });
        events
    }

    fn timeline_event_sets_typed(&self) -> SubtrActorResult<ReplayStatsTimelineEvents> {
        Ok(ReplayStatsTimelineEvents {
            timeline: self.timeline_events_typed()?,
            core_player: self.module_player_events(
                "core",
                "player_events",
                parse_core_player_stats_event,
            )?,
            core_team: self.module_player_events(
                "core",
                "team_events",
                parse_core_team_stats_event,
            )?,
            possession: self.module_player_events(
                "possession",
                "events",
                parse_possession_event,
            )?,
            pressure: self.module_player_events("pressure", "events", parse_pressure_event)?,
            territorial_pressure: self.module_player_events(
                "territorial_pressure",
                "events",
                parse_territorial_pressure_event,
            )?,
            movement: self.module_player_events("movement", "events", parse_movement_event)?,
            positioning: self.module_player_events(
                "positioning",
                "events",
                parse_positioning_event,
            )?,
            rotation_player: self.module_player_events(
                "rotation",
                "player_events",
                parse_rotation_player_event,
            )?,
            rotation_team: self.module_player_events(
                "rotation",
                "team_events",
                parse_rotation_team_event,
            )?,
            mechanics: self.mechanic_events_typed()?,
            goal_context: self.module_player_events(
                "core",
                "goal_context",
                parse_goal_context_event,
            )?,
            backboard: self.module_player_events("backboard", "events", parse_backboard_event)?,
            ceiling_shot: self.module_player_events(
                "ceiling_shot",
                "events",
                parse_ceiling_shot_event,
            )?,
            wall_aerial: self.module_player_events(
                "wall_aerial",
                "events",
                parse_wall_aerial_event,
            )?,
            wall_aerial_shot: self.module_player_events(
                "wall_aerial_shot",
                "events",
                parse_wall_aerial_shot_event,
            )?,
            center: self.module_player_events("center", "events", parse_center_event)?,
            flick: self.module_player_events("flick", "events", parse_flick_event)?,
            musty_flick: self.module_player_events(
                "musty_flick",
                "events",
                parse_musty_flick_event,
            )?,
            dodge_reset: self.module_player_events(
                "dodge_reset",
                "events",
                parse_dodge_reset_event,
            )?,
            double_tap: self.module_player_events(
                "double_tap",
                "events",
                parse_double_tap_event,
            )?,
            one_timer: self.module_player_events("one_timer", "events", parse_one_timer_event)?,
            fifty_fifty: self.module_player_events(
                "fifty_fifty",
                "events",
                parse_fifty_fifty_event,
            )?,
            pass: self.module_player_events("pass", "events", parse_pass_event)?,
            pass_last_completed: self.module_player_events(
                "pass",
                "last_completed_events",
                parse_pass_last_completed_event,
            )?,
            ball_carry: self.module_player_events(
                "ball_carry",
                "events",
                parse_ball_carry_event,
            )?,
            goal_tags: self.goal_tag_events_typed()?,
            rush: self.module_typed_array("rush", "events")?,
            speed_flip: self.module_player_events(
                "speed_flip",
                "events",
                parse_speed_flip_event,
            )?,
            half_flip: self.module_player_events("half_flip", "events", parse_half_flip_event)?,
            half_volley: self.module_player_events(
                "half_volley",
                "events",
                parse_half_volley_event,
            )?,
            wavedash: self.module_player_events("wavedash", "events", parse_wavedash_event)?,
            whiff: self.module_player_events("whiff", "events", parse_whiff_event)?,
            powerslide: self.module_player_events(
                "powerslide",
                "events",
                parse_powerslide_event,
            )?,
            touch: self.module_player_events("touch", "events", parse_touch_stats_event)?,
            touch_ball_movement: self.module_player_events(
                "touch",
                "ball_movement_events",
                parse_touch_ball_movement_event,
            )?,
            touch_last_touch: self.module_player_events(
                "touch",
                "last_touch_events",
                parse_touch_last_touch_event,
            )?,
            boost_pickups: self.module_player_events(
                "boost",
                "events",
                parse_boost_pickup_comparison_event,
            )?,
            boost_ledger: self.module_player_events(
                "boost",
                "ledger_events",
                parse_boost_ledger_event,
            )?,
            boost_state: self.module_player_events(
                "boost",
                "state_events",
                parse_boost_state_event,
            )?,
            bump: self.module_player_events("bump", "events", parse_bump_event)?,
        })
    }

    fn timeline_event_sets_value(&self) -> SubtrActorResult<Value> {
        let mut events = Map::new();
        events.insert("timeline".to_owned(), Value::Array(self.timeline_events()));
        events.insert(
            "core_player".to_owned(),
            Value::Array(self.module_array("core", "player_events")),
        );
        events.insert(
            "core_team".to_owned(),
            Value::Array(self.module_array("core", "team_events")),
        );
        events.insert(
            "possession".to_owned(),
            Value::Array(self.module_array("possession", "events")),
        );
        events.insert(
            "pressure".to_owned(),
            Value::Array(self.module_array("pressure", "events")),
        );
        events.insert(
            "territorial_pressure".to_owned(),
            Value::Array(self.module_array("territorial_pressure", "events")),
        );
        events.insert(
            "movement".to_owned(),
            Value::Array(self.module_array("movement", "events")),
        );
        events.insert(
            "positioning".to_owned(),
            Value::Array(self.module_array("positioning", "events")),
        );
        events.insert(
            "rotation_player".to_owned(),
            Value::Array(self.module_array("rotation", "player_events")),
        );
        events.insert(
            "rotation_team".to_owned(),
            Value::Array(self.module_array("rotation", "team_events")),
        );
        events.insert(
            "mechanics".to_owned(),
            serialize_to_json_value(&self.mechanic_events_typed()?)?,
        );
        events.insert(
            "backboard".to_owned(),
            Value::Array(self.module_array("backboard", "events")),
        );
        events.insert(
            "ceiling_shot".to_owned(),
            Value::Array(self.module_array("ceiling_shot", "events")),
        );
        events.insert(
            "wall_aerial".to_owned(),
            Value::Array(self.module_array("wall_aerial", "events")),
        );
        events.insert(
            "wall_aerial_shot".to_owned(),
            Value::Array(self.module_array("wall_aerial_shot", "events")),
        );
        events.insert(
            "center".to_owned(),
            Value::Array(self.module_array("center", "events")),
        );
        events.insert(
            "double_tap".to_owned(),
            Value::Array(self.module_array("double_tap", "events")),
        );
        events.insert(
            "one_timer".to_owned(),
            Value::Array(self.module_array("one_timer", "events")),
        );
        events.insert(
            "pass".to_owned(),
            Value::Array(self.module_array("pass", "events")),
        );
        events.insert(
            "goal_tags".to_owned(),
            Value::Array(self.goal_tag_events_value()),
        );
        events.insert(
            "fifty_fifty".to_owned(),
            Value::Array(self.module_array("fifty_fifty", "events")),
        );
        events.insert(
            "rush".to_owned(),
            Value::Array(self.module_array("rush", "events")),
        );
        events.insert(
            "speed_flip".to_owned(),
            Value::Array(self.module_array("speed_flip", "events")),
        );
        events.insert(
            "half_flip".to_owned(),
            Value::Array(self.module_array("half_flip", "events")),
        );
        events.insert(
            "half_volley".to_owned(),
            Value::Array(self.module_array("half_volley", "events")),
        );
        events.insert(
            "wavedash".to_owned(),
            Value::Array(self.module_array("wavedash", "events")),
        );
        events.insert(
            "whiff".to_owned(),
            Value::Array(self.module_array("whiff", "events")),
        );
        events.insert(
            "touch".to_owned(),
            Value::Array(self.module_array("touch", "events")),
        );
        events.insert(
            "touch_ball_movement".to_owned(),
            Value::Array(self.module_array("touch", "ball_movement_events")),
        );
        events.insert(
            "touch_last_touch".to_owned(),
            Value::Array(self.module_array("touch", "last_touch_events")),
        );
        events.insert(
            "boost_pickups".to_owned(),
            Value::Array(self.module_array("boost", "events")),
        );
        events.insert(
            "boost_ledger".to_owned(),
            Value::Array(self.module_array("boost", "ledger_events")),
        );
        events.insert(
            "boost_state".to_owned(),
            Value::Array(self.module_array("boost", "state_events")),
        );
        events.insert(
            "bump".to_owned(),
            Value::Array(self.module_array("bump", "events")),
        );
        Ok(Value::Object(events))
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

    fn timeline_frame_value(&self, frame: &StatsSnapshotFrame) -> SubtrActorResult<Value> {
        let mut timeline = Map::new();
        timeline.insert(
            "frame_number".to_owned(),
            serialize_to_json_value(&frame.frame_number)?,
        );
        timeline.insert("time".to_owned(), serialize_to_json_value(&frame.time)?);
        timeline.insert("dt".to_owned(), serialize_to_json_value(&frame.dt)?);
        timeline.insert(
            "seconds_remaining".to_owned(),
            serialize_to_json_value(&frame.seconds_remaining)?,
        );
        timeline.insert(
            "game_state".to_owned(),
            serialize_to_json_value(&frame.game_state)?,
        );
        timeline.insert(
            "ball_has_been_hit".to_owned(),
            serialize_to_json_value(&frame.ball_has_been_hit)?,
        );
        timeline.insert(
            "kickoff_countdown_time".to_owned(),
            serialize_to_json_value(&frame.kickoff_countdown_time)?,
        );
        timeline.insert(
            "gameplay_phase".to_owned(),
            serialize_to_json_value(&frame.gameplay_phase)?,
        );
        timeline.insert(
            "is_live_play".to_owned(),
            serialize_to_json_value(&frame.is_live_play)?,
        );
        timeline.insert(
            "fifty_fifty".to_owned(),
            self.frame_stats_or_default::<FiftyFiftyStats>(frame, "fifty_fifty"),
        );
        timeline.insert(
            "possession".to_owned(),
            self.frame_stats_or_default::<PossessionStats>(frame, "possession"),
        );
        timeline.insert(
            "pressure".to_owned(),
            self.frame_stats_or_default::<PressureStats>(frame, "pressure"),
        );
        timeline.insert(
            "territorial_pressure".to_owned(),
            self.frame_stats_or_default::<TerritorialPressureStats>(frame, "territorial_pressure"),
        );
        timeline.insert(
            "rush".to_owned(),
            self.frame_stats_or_default::<RushStats>(frame, "rush"),
        );
        timeline.insert(
            "team_zero".to_owned(),
            self.timeline_team_value(frame, "team_zero")?,
        );
        timeline.insert(
            "team_one".to_owned(),
            self.timeline_team_value(frame, "team_one")?,
        );
        timeline.insert(
            "players".to_owned(),
            Value::Array(
                self.replay_meta
                    .player_order()
                    .map(|player| self.timeline_player_value(frame, player))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(timeline))
    }

    pub(crate) fn replay_stats_frame(
        &self,
        frame: &StatsSnapshotFrame,
    ) -> SubtrActorResult<ReplayStatsFrame> {
        Ok(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: frame.game_state,
            ball_has_been_hit: frame.ball_has_been_hit,
            kickoff_countdown_time: frame.kickoff_countdown_time,
            gameplay_phase: frame.gameplay_phase,
            is_live_play: frame.is_live_play,
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
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        let is_team_zero = team_key == "team_zero";
        Ok(TeamStatsSnapshot {
            fifty_fifty: self
                .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                .for_team(is_team_zero),
            possession: self
                .frame_stats_or_default_typed::<PossessionStats>(frame, "possession")?
                .for_team(is_team_zero),
            pressure: self
                .frame_stats_or_default_typed::<PressureStats>(frame, "pressure")?
                .for_team(is_team_zero),
            territorial_pressure: self
                .frame_stats_or_default_typed::<TerritorialPressureStats>(
                    frame,
                    "territorial_pressure",
                )?
                .for_team(is_team_zero),
            rotation: self.frame_team_stat_or_default_typed(frame, "rotation", team_key)?,
            rush: self
                .frame_stats_or_default_typed::<RushStats>(frame, "rush")?
                .for_team(is_team_zero),
            core: self.frame_team_stat_or_default_typed(frame, "core", team_key)?,
            backboard: self.frame_team_stat_or_default_typed(frame, "backboard", team_key)?,
            double_tap: self.frame_team_stat_or_default_typed(frame, "double_tap", team_key)?,
            one_timer: self.frame_team_stat_or_default_typed(frame, "one_timer", team_key)?,
            pass: self.frame_team_stat_or_default_typed(frame, "pass", team_key)?,
            ball_carry: self.frame_team_stat_or_default_typed(frame, "ball_carry", team_key)?,
            air_dribble: self.frame_team_stat_or_default_typed(frame, "air_dribble", team_key)?,
            boost: self.frame_team_stat_or_default_typed(frame, "boost", team_key)?,
            bump: self.frame_team_stat_or_default_typed(frame, "bump", team_key)?,
            half_volley: self.frame_team_stat_or_default_typed(frame, "half_volley", team_key)?,
            movement: self.frame_team_stat_or_default_typed(frame, "movement", team_key)?,
            powerslide: self.frame_team_stat_or_default_typed(frame, "powerslide", team_key)?,
            demo: self.frame_team_stat_or_default_typed(frame, "demo", team_key)?,
        })
    }

    fn replay_player_stats(
        &self,
        frame: &StatsSnapshotFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<PlayerStatsSnapshot> {
        let player_key = player_info_key(player)?;
        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: self.is_team_zero_player(player),
            core: self.frame_core_player_stat_or_default_by_key(frame, &player_key)?,
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
            wall_aerial: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wall_aerial",
                &player_key,
            )?,
            wall_aerial_shot: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wall_aerial_shot",
                &player_key,
            )?,
            double_tap: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "double_tap",
                &player_key,
            )?,
            one_timer: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "one_timer",
                &player_key,
            )?,
            pass: self.frame_player_stat_or_default_typed_by_key(frame, "pass", &player_key)?,
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
            half_flip: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "half_flip",
                &player_key,
            )?,
            wavedash: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wavedash",
                &player_key,
            )?,
            touch: if frame.modules.contains_key("touch") {
                self.frame_player_stat_or_default_with_by_key(frame, "touch", &player_key, || {
                    TouchStats::default().with_complete_labeled_touch_counts()
                })?
            } else {
                self.frame_player_stat_or_default_typed_by_key(frame, "touch", &player_key)?
            },
            whiff: self.frame_player_stat_or_default_typed_by_key(frame, "whiff", &player_key)?,
            flick: self.frame_player_stat_or_default_typed_by_key(frame, "flick", &player_key)?,
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
            air_dribble: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "air_dribble",
                &player_key,
            )?,
            boost: self.frame_player_stat_or_default_typed_by_key(frame, "boost", &player_key)?,
            bump: self.frame_player_stat_or_default_typed_by_key(frame, "bump", &player_key)?,
            half_volley: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "half_volley",
                &player_key,
            )?,
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
            rotation: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "rotation",
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

    fn timeline_team_value(
        &self,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<Value> {
        let is_team_zero = team_key == "team_zero";
        let mut team = Map::new();
        team.insert(
            "fifty_fifty".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "possession".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<PossessionStats>(frame, "possession")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "pressure".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<PressureStats>(frame, "pressure")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "territorial_pressure".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<TerritorialPressureStats>(
                        frame,
                        "territorial_pressure",
                    )?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "rotation".to_owned(),
            self.frame_team_stat_or_default::<RotationTeamStats>(frame, "rotation", team_key),
        );
        team.insert(
            "rush".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<RushStats>(frame, "rush")?
                    .for_team(is_team_zero),
            )?,
        );
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
            "one_timer".to_owned(),
            self.frame_team_stat_or_default::<OneTimerTeamStats>(frame, "one_timer", team_key),
        );
        team.insert(
            "pass".to_owned(),
            self.frame_team_stat_or_default::<PassTeamStats>(frame, "pass", team_key),
        );
        team.insert(
            "ball_carry".to_owned(),
            self.frame_team_stat_or_default::<BallCarryStats>(frame, "ball_carry", team_key),
        );
        team.insert(
            "air_dribble".to_owned(),
            self.frame_team_stat_or_default::<AirDribbleStats>(frame, "air_dribble", team_key),
        );
        team.insert(
            "boost".to_owned(),
            self.frame_team_stat_or_default::<BoostStats>(frame, "boost", team_key),
        );
        team.insert(
            "bump".to_owned(),
            self.frame_team_stat_or_default::<BumpTeamStats>(frame, "bump", team_key),
        );
        team.insert(
            "half_volley".to_owned(),
            self.frame_team_stat_or_default::<HalfVolleyTeamStats>(frame, "half_volley", team_key),
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

    fn timeline_player_value(
        &self,
        frame: &StatsSnapshotFrame,
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
            "wall_aerial".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialStats>(
                frame,
                "wall_aerial",
                &player_key,
            )?,
        );
        player_value.insert(
            "wall_aerial_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialShotStats>(
                frame,
                "wall_aerial_shot",
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
            "one_timer".to_owned(),
            self.frame_player_stat_or_default_by_key::<OneTimerPlayerStats>(
                frame,
                "one_timer",
                &player_key,
            )?,
        );
        player_value.insert(
            "pass".to_owned(),
            self.frame_player_stat_or_default_by_key::<PassPlayerStats>(
                frame,
                "pass",
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
            "half_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfFlipStats>(
                frame,
                "half_flip",
                &player_key,
            )?,
        );
        player_value.insert(
            "half_volley".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfVolleyPlayerStats>(
                frame,
                "half_volley",
                &player_key,
            )?,
        );
        player_value.insert(
            "wavedash".to_owned(),
            self.frame_player_stat_or_default_by_key::<WavedashStats>(
                frame,
                "wavedash",
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
            "whiff".to_owned(),
            self.frame_player_stat_or_default_by_key::<WhiffStats>(frame, "whiff", &player_key)?,
        );
        player_value.insert(
            "flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<FlickStats>(frame, "flick", &player_key)?,
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
            "air_dribble".to_owned(),
            self.frame_player_stat_or_default_by_key::<AirDribbleStats>(
                frame,
                "air_dribble",
                &player_key,
            )?,
        );
        player_value.insert(
            "boost".to_owned(),
            self.frame_player_stat_or_default_by_key::<BoostStats>(frame, "boost", &player_key)?,
        );
        player_value.insert(
            "bump".to_owned(),
            self.frame_player_stat_or_default_by_key::<BumpPlayerStats>(
                frame,
                "bump",
                &player_key,
            )?,
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
            "rotation".to_owned(),
            self.frame_player_stat_or_default_by_key::<RotationPlayerStats>(
                frame,
                "rotation",
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

    fn frame_stats_or_default<T>(&self, frame: &StatsSnapshotFrame, module_name: &str) -> Value
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
        frame: &StatsSnapshotFrame,
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
        frame: &StatsSnapshotFrame,
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
        frame: &StatsSnapshotFrame,
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
        frame: &StatsSnapshotFrame,
        module_name: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_stats_or_default::<T>(frame, module_name))
    }

    fn frame_team_stat_or_default_typed<T>(
        &self,
        frame: &StatsSnapshotFrame,
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
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        self.frame_player_stat_or_default_with_by_key(frame, module_name, player_key, T::default)
    }

    fn frame_core_player_stat_or_default_by_key(
        &self,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<CorePlayerStats> {
        decode_core_player_stats_value(self.frame_player_stat_or_value_by_key(
            frame,
            "core",
            player_key,
            default_json_value::<CorePlayerStats>(),
        )?)
    }

    fn frame_player_stat_or_default_with_by_key<T, F>(
        &self,
        frame: &StatsSnapshotFrame,
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
