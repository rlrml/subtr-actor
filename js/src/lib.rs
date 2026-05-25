#![allow(clippy::result_large_err)]
#![allow(dead_code)]

use js_sys::{Array, Function, Object, Reflect, Uint8Array};
use serde_json::Value;
use subtr_actor::{
    collector::replay_data::{ReplayData, ReplayDataCollector},
    collector::CallbackCollector,
    Collector, FrameRateDecorator, NDArrayCollector, ReplayProcessor, ResolvedBoostPadCollector,
    StatsCollector, StatsTimelineEventCollector, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log!("subtr-actor WASM bindings loaded");
}

// Default feature adders (same as Python bindings)
const DEFAULT_GLOBAL_FEATURE_ADDERS: &[&str] = &["BallRigidBody"];
const DEFAULT_PLAYER_FEATURE_ADDERS: &[&str] = &["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"];
const DEFAULT_STATS_TIMELINE_FRAME_CHUNK_BYTES: usize = 32 * 1024 * 1024;
const TEAM_STATS_MODULE_FIELDS: &[&str] = &[
    "fifty_fifty",
    "possession",
    "pressure",
    "rotation",
    "rush",
    "core",
    "backboard",
    "double_tap",
    "one_timer",
    "pass",
    "ball_carry",
    "air_dribble",
    "boost",
    "bump",
    "half_volley",
    "movement",
    "powerslide",
    "demo",
];
const PLAYER_STATS_MODULE_FIELDS: &[&str] = &[
    "core",
    "backboard",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "double_tap",
    "one_timer",
    "pass",
    "fifty_fifty",
    "speed_flip",
    "half_flip",
    "half_volley",
    "wavedash",
    "touch",
    "whiff",
    "flick",
    "musty_flick",
    "dodge_reset",
    "ball_carry",
    "air_dribble",
    "boost",
    "bump",
    "movement",
    "positioning",
    "rotation",
    "powerslide",
    "demo",
];
const EVENT_DERIVED_BOOST_FIELDS: &[&str] = &[
    "tracked_time",
    "boost_integral",
    "time_zero_boost",
    "time_hundred_boost",
    "time_boost_0_25",
    "time_boost_25_50",
    "time_boost_50_75",
    "time_boost_75_100",
    "amount_collected",
    "amount_collected_inactive",
    "big_pads_collected_inactive",
    "small_pads_collected_inactive",
    "amount_stolen",
    "big_pads_collected",
    "small_pads_collected",
    "big_pads_stolen",
    "small_pads_stolen",
    "amount_collected_big",
    "amount_stolen_big",
    "amount_collected_small",
    "amount_stolen_small",
    "amount_respawned",
    "overfill_total",
    "overfill_from_stolen",
    "amount_used",
    "amount_used_while_grounded",
    "amount_used_while_airborne",
    "amount_used_while_supersonic",
    "labeled_amounts",
    "labeled_counts",
];
const EVENT_DERIVED_CORE_TEAM_FIELDS: &[&str] = &[
    "score",
    "goals",
    "assists",
    "saves",
    "shots",
    "kickoff_goal_count",
    "short_goal_count",
    "medium_goal_count",
    "long_goal_count",
    "goal_times",
    "counter_attack_goal_count",
    "sustained_pressure_goal_count",
    "other_buildup_goal_count",
    "goal_ball_air_time_sample_count",
    "cumulative_goal_ball_air_time",
    "last_goal_ball_air_time",
    "goal_ball_air_times",
];
const EVENT_DERIVED_CORE_PLAYER_FIELDS: &[&str] = &[
    "score",
    "goals",
    "assists",
    "saves",
    "shots",
    "goals_conceded_while_last_defender",
    "goals_for_while_most_back",
    "goals_against_while_most_back",
    "goal_against_boost_sample_count",
    "cumulative_boost_on_goals_against",
    "last_boost_on_goal_against",
    "goal_against_boost_leadup_sample_count",
    "cumulative_average_boost_in_goal_against_leadup",
    "cumulative_min_boost_in_goal_against_leadup",
    "last_average_boost_in_goal_against_leadup",
    "last_min_boost_in_goal_against_leadup",
    "goal_against_position_sample_count",
    "cumulative_goal_against_position_x",
    "cumulative_goal_against_position_y",
    "cumulative_goal_against_position_z",
    "last_goal_against_position",
    "scoring_goal_last_touch_position_sample_count",
    "cumulative_scoring_goal_last_touch_position_x",
    "cumulative_scoring_goal_last_touch_position_y",
    "cumulative_scoring_goal_last_touch_position_z",
    "last_scoring_goal_last_touch_position",
    "kickoff_goal_count",
    "short_goal_count",
    "medium_goal_count",
    "long_goal_count",
    "goal_times",
    "counter_attack_goal_count",
    "sustained_pressure_goal_count",
    "other_buildup_goal_count",
    "goal_ball_air_time_sample_count",
    "cumulative_goal_ball_air_time",
    "last_goal_ball_air_time",
    "goal_ball_air_times",
];
const EVENT_DERIVED_POSSESSION_TEAM_FIELDS: &[&str] = &[
    "tracked_time",
    "possession_time",
    "opponent_possession_time",
    "neutral_time",
    "labeled_time",
];
const EVENT_DERIVED_PRESSURE_TEAM_FIELDS: &[&str] = &[
    "tracked_time",
    "defensive_half_time",
    "offensive_half_time",
    "neutral_time",
    "labeled_time",
];
const EVENT_DERIVED_MOVEMENT_FIELDS: &[&str] = &[
    "tracked_time",
    "total_distance",
    "speed_integral",
    "time_slow_speed",
    "time_boost_speed",
    "time_supersonic_speed",
    "time_on_ground",
    "time_low_air",
    "time_high_air",
    "labeled_tracked_time",
];
const EVENT_DERIVED_POSITIONING_FIELDS: &[&str] = &[
    "active_game_time",
    "tracked_time",
    "sum_distance_to_teammates",
    "sum_distance_to_ball",
    "sum_distance_to_ball_has_possession",
    "time_has_possession",
    "sum_distance_to_ball_no_possession",
    "time_no_possession",
    "time_demolished",
    "time_no_teammates",
    "time_most_back",
    "time_most_forward",
    "time_mid_role",
    "time_other_role",
    "time_defensive_third",
    "time_neutral_third",
    "time_offensive_third",
    "time_defensive_half",
    "time_offensive_half",
    "time_closest_to_ball",
    "time_farthest_from_ball",
    "time_behind_ball",
    "time_level_with_ball",
    "time_in_front_of_ball",
    "times_caught_ahead_of_play_on_conceded_goals",
];
const EVENT_DERIVED_ROTATION_PLAYER_FIELDS: &[&str] = &[
    "active_game_time",
    "tracked_time",
    "time_first_man",
    "time_second_man",
    "time_third_man",
    "time_ambiguous_role",
    "time_behind_play",
    "time_level_with_play",
    "time_ahead_of_play",
    "became_first_man_count",
    "lost_first_man_count",
    "current_role_state",
    "current_depth_state",
];
const EVENT_DERIVED_ROTATION_TEAM_FIELDS: &[&str] =
    &["first_man_changes_for_team", "rotation_count"];
const EVENT_DERIVED_BACKBOARD_PLAYER_FIELDS: &[&str] = &[
    "count",
    "is_last_backboard",
    "last_backboard_time",
    "last_backboard_frame",
    "time_since_last_backboard",
    "frames_since_last_backboard",
];
const EVENT_DERIVED_BACKBOARD_TEAM_FIELDS: &[&str] = &["count"];
const EVENT_DERIVED_DOUBLE_TAP_PLAYER_FIELDS: &[&str] = &[
    "count",
    "is_last_double_tap",
    "last_double_tap_time",
    "last_double_tap_frame",
    "time_since_last_double_tap",
    "frames_since_last_double_tap",
];
const EVENT_DERIVED_DOUBLE_TAP_TEAM_FIELDS: &[&str] = &["count"];
const EVENT_DERIVED_CEILING_SHOT_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_ceiling_shot",
    "last_ceiling_shot_time",
    "last_ceiling_shot_frame",
    "time_since_last_ceiling_shot",
    "frames_since_last_ceiling_shot",
    "last_confidence",
    "best_confidence",
    "cumulative_confidence",
    "labeled_event_counts",
];
const EVENT_DERIVED_ONE_TIMER_PLAYER_FIELDS: &[&str] = &[
    "count",
    "total_ball_speed",
    "fastest_ball_speed",
    "total_pass_distance",
    "is_last_one_timer",
    "last_one_timer_time",
    "last_one_timer_frame",
    "time_since_last_one_timer",
    "frames_since_last_one_timer",
];
const EVENT_DERIVED_ONE_TIMER_TEAM_FIELDS: &[&str] =
    &["count", "total_ball_speed", "fastest_ball_speed"];
const EVENT_DERIVED_HALF_VOLLEY_PLAYER_FIELDS: &[&str] = &[
    "count",
    "total_ball_speed",
    "fastest_ball_speed",
    "is_last_half_volley",
    "last_half_volley_time",
    "last_half_volley_frame",
    "time_since_last_half_volley",
    "frames_since_last_half_volley",
];
const EVENT_DERIVED_HALF_VOLLEY_TEAM_FIELDS: &[&str] =
    &["count", "total_ball_speed", "fastest_ball_speed"];
const EVENT_DERIVED_PASS_PLAYER_FIELDS: &[&str] = &[
    "completed_pass_count",
    "received_pass_count",
    "total_pass_distance",
    "total_pass_advance",
    "longest_pass_distance",
    "is_last_completed_pass",
    "last_completed_pass_time",
    "last_completed_pass_frame",
    "time_since_last_completed_pass",
    "frames_since_last_completed_pass",
];
const EVENT_DERIVED_PASS_TEAM_FIELDS: &[&str] = &[
    "completed_pass_count",
    "total_pass_distance",
    "total_pass_advance",
    "longest_pass_distance",
];
const EVENT_DERIVED_BALL_CARRY_FIELDS: &[&str] = &[
    "carry_count",
    "total_carry_time",
    "total_straight_line_distance",
    "total_path_distance",
    "longest_carry_time",
    "furthest_carry_distance",
    "fastest_carry_speed",
    "carry_speed_sum",
    "average_horizontal_gap_sum",
    "average_vertical_gap_sum",
    "labeled_event_counts",
];
const EVENT_DERIVED_AIR_DRIBBLE_FIELDS: &[&str] = &[
    "count",
    "ground_to_air_count",
    "wall_to_air_count",
    "total_touch_count",
    "max_touch_count",
    "total_time",
    "total_straight_line_distance",
    "total_path_distance",
    "longest_time",
    "furthest_distance",
    "fastest_speed",
    "speed_sum",
    "average_horizontal_gap_sum",
    "average_vertical_gap_sum",
    "labeled_event_counts",
];
const EVENT_DERIVED_WALL_AERIAL_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_wall_aerial",
    "last_wall_aerial_time",
    "last_wall_aerial_frame",
    "time_since_last_wall_aerial",
    "frames_since_last_wall_aerial",
    "last_confidence",
    "best_confidence",
    "cumulative_confidence",
    "cumulative_setup_duration",
    "cumulative_takeoff_to_touch_time",
    "cumulative_touch_height",
];
const EVENT_DERIVED_WALL_AERIAL_SHOT_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_wall_aerial_shot",
    "last_wall_aerial_shot_time",
    "last_wall_aerial_shot_frame",
    "time_since_last_wall_aerial_shot",
    "frames_since_last_wall_aerial_shot",
    "last_confidence",
    "best_confidence",
    "cumulative_confidence",
    "cumulative_takeoff_to_shot_time",
    "cumulative_shot_height",
];
const EVENT_DERIVED_FLICK_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_flick",
    "last_flick_time",
    "last_flick_frame",
    "time_since_last_flick",
    "frames_since_last_flick",
    "last_confidence",
    "best_confidence",
    "cumulative_confidence",
    "cumulative_setup_duration",
    "cumulative_ball_speed_change",
    "labeled_event_counts",
];
const EVENT_DERIVED_MUSTY_FLICK_FIELDS: &[&str] = &[
    "count",
    "aerial_count",
    "high_confidence_count",
    "is_last_musty",
    "last_musty_time",
    "last_musty_frame",
    "time_since_last_musty",
    "frames_since_last_musty",
    "last_confidence",
    "best_confidence",
    "cumulative_confidence",
    "labeled_event_counts",
];
const EVENT_DERIVED_DODGE_RESET_FIELDS: &[&str] = &["count", "on_ball_count"];
const EVENT_DERIVED_POWERSLIDE_FIELDS: &[&str] = &["total_duration", "press_count"];
const EVENT_DERIVED_TOUCH_FIELDS: &[&str] = &[
    "touch_count",
    "control_touch_count",
    "medium_hit_count",
    "hard_hit_count",
    "aerial_touch_count",
    "high_aerial_touch_count",
    "wall_touch_count",
    "is_last_touch",
    "last_touch_time",
    "last_touch_frame",
    "time_since_last_touch",
    "frames_since_last_touch",
    "last_ball_speed_change",
    "max_ball_speed_change",
    "cumulative_ball_speed_change",
    "total_ball_travel_distance",
    "total_ball_advance_distance",
    "total_ball_retreat_distance",
    "labeled_touch_counts",
];
const EVENT_DERIVED_RUSH_TEAM_FIELDS: &[&str] = &[
    "count",
    "two_v_one_count",
    "two_v_two_count",
    "two_v_three_count",
    "three_v_one_count",
    "three_v_two_count",
    "three_v_three_count",
];
const EVENT_DERIVED_BUMP_PLAYER_FIELDS: &[&str] = &[
    "bumps_inflicted",
    "bumps_taken",
    "team_bumps_inflicted",
    "team_bumps_taken",
    "last_bump_time",
    "last_bump_frame",
    "last_bump_strength",
    "max_bump_strength",
    "cumulative_bump_strength",
];
const EVENT_DERIVED_BUMP_TEAM_FIELDS: &[&str] = &["bumps_inflicted", "team_bumps_inflicted"];
const EVENT_DERIVED_FIFTY_FIFTY_PLAYER_FIELDS: &[&str] = &[
    "count",
    "wins",
    "losses",
    "neutral_outcomes",
    "kickoff_count",
    "kickoff_wins",
    "kickoff_losses",
    "kickoff_neutral_outcomes",
    "possession_after_count",
    "kickoff_possession_after_count",
    "labeled_event_counts",
];
const EVENT_DERIVED_FIFTY_FIFTY_TEAM_FIELDS: &[&str] = &[
    "count",
    "wins",
    "losses",
    "neutral_outcomes",
    "kickoff_count",
    "kickoff_wins",
    "kickoff_losses",
    "kickoff_neutral_outcomes",
    "possession_after_count",
    "opponent_possession_after_count",
    "neutral_possession_after_count",
    "kickoff_possession_after_count",
    "kickoff_opponent_possession_after_count",
    "kickoff_neutral_possession_after_count",
];
const EVENT_DERIVED_DEMO_PLAYER_FIELDS: &[&str] = &["demos_inflicted", "demos_taken"];
const EVENT_DERIVED_DEMO_TEAM_FIELDS: &[&str] = &["demos_inflicted"];
const EVENT_DERIVED_SPEED_FLIP_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_speed_flip",
    "last_speed_flip_time",
    "last_speed_flip_frame",
    "time_since_last_speed_flip",
    "frames_since_last_speed_flip",
    "last_quality",
    "best_quality",
    "cumulative_quality",
    "labeled_event_counts",
];
const EVENT_DERIVED_HALF_FLIP_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_half_flip",
    "last_half_flip_time",
    "last_half_flip_frame",
    "time_since_last_half_flip",
    "frames_since_last_half_flip",
    "last_quality",
    "best_quality",
    "cumulative_quality",
    "labeled_event_counts",
];
const EVENT_DERIVED_WAVEDASH_FIELDS: &[&str] = &[
    "count",
    "high_confidence_count",
    "is_last_wavedash",
    "last_wavedash_time",
    "last_wavedash_frame",
    "time_since_last_wavedash",
    "frames_since_last_wavedash",
    "last_quality",
    "best_quality",
    "cumulative_quality",
    "labeled_event_counts",
];
const EVENT_DERIVED_WHIFF_FIELDS: &[&str] = &[
    "whiff_count",
    "beaten_to_ball_count",
    "grounded_whiff_count",
    "aerial_whiff_count",
    "dodge_whiff_count",
    "is_last_whiff",
    "last_whiff_time",
    "last_whiff_frame",
    "time_since_last_whiff",
    "frames_since_last_whiff",
    "last_closest_approach_distance",
    "best_closest_approach_distance",
    "cumulative_closest_approach_distance",
    "labeled_whiff_counts",
];

fn parse_replay_from_data(data: &[u8]) -> Result<boxcars::Replay, JsValue> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Failed to parse replay: {e}")))
}

fn get_total_frames(replay: &boxcars::Replay) -> Result<usize, JsValue> {
    replay
        .network_frames
        .as_ref()
        .map(|network_frames| network_frames.frames.len())
        .ok_or_else(|| JsValue::from_str("Replay has no network frames"))
}

fn emit_progress(
    callback: &Function,
    stage: &str,
    processed_frames: usize,
    total_frames: usize,
) -> SubtrActorResult<()> {
    let progress = if total_frames == 0 {
        1.0
    } else {
        processed_frames as f64 / total_frames as f64
    };
    let payload = serde_wasm_bindgen::to_value(&serde_json::json!({
        "stage": stage,
        "processedFrames": processed_frames,
        "totalFrames": total_frames,
        "progress": progress,
    }))
    .map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "Failed to serialize progress payload: {error}"
        )))
    })?;

    callback.call1(&JsValue::NULL, &payload).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
            error
                .as_string()
                .unwrap_or_else(|| "Progress callback threw a non-string error".to_string()),
        ))
    })?;
    Ok(())
}

fn emit_stage_progress(callback: &Function, stage: &str, progress: f64) -> SubtrActorResult<()> {
    let payload = serde_wasm_bindgen::to_value(&serde_json::json!({
        "stage": stage,
        "progress": progress.clamp(0.0, 1.0),
    }))
    .map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "Failed to serialize progress payload: {error}"
        )))
    })?;

    callback.call1(&JsValue::NULL, &payload).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
            error
                .as_string()
                .unwrap_or_else(|| "Progress callback threw a non-string error".to_string()),
        ))
    })?;
    Ok(())
}

fn emit_stats_timeline_progress(callback: &Function, progress: f64) -> SubtrActorResult<()> {
    emit_stage_progress(callback, "stats-timeline", progress)
}

fn collect_replay_data_with_optional_progress(
    replay: &boxcars::Replay,
    progress: Option<(&Function, usize)>,
) -> Result<ReplayData, JsValue> {
    let total_frames = get_total_frames(replay)?;
    let mut processor = ReplayProcessor::new(replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut boost_pad_collector = ResolvedBoostPadCollector::new();
    let mut last_reported_frames = 0usize;
    let mut progress_collector = progress
        .map(|(callback, frame_interval)| {
            emit_progress(callback, "processing", 0, total_frames)?;
            Ok::<_, SubtrActorError>(CallbackCollector::with_frame_interval(
                |_frame, frame_number, _current_time| {
                    last_reported_frames = frame_number + 1;
                    emit_progress(callback, "processing", last_reported_frames, total_frames)
                },
                frame_interval.max(1),
            ))
        })
        .transpose()
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    let mut collectors: Vec<&mut dyn Collector> =
        vec![&mut replay_data_collector, &mut boost_pad_collector];
    if let Some(progress_collector) = progress_collector.as_mut() {
        collectors.push(progress_collector);
    }

    processor
        .process_all(&mut collectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    if let Some((callback, _)) = progress {
        if last_reported_frames < total_frames {
            emit_progress(callback, "processing", total_frames, total_frames).map_err(|error| {
                JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
            })?;
        }
    }

    replay_data_collector
        .into_replay_data_with_boost_pads(processor, boost_pad_collector.into_resolved_boost_pads())
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))
}

fn collect_replay_bundle_with_optional_progress(
    replay: &boxcars::Replay,
    progress: Option<(&Function, usize)>,
) -> Result<(ReplayData, subtr_actor::ReplayStatsTimelineScaffold), JsValue> {
    let total_frames = get_total_frames(replay)?;
    let mut processor = ReplayProcessor::new(replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut stats_collector = StatsTimelineEventCollector::new();
    let mut boost_pad_collector = ResolvedBoostPadCollector::new();
    let mut last_reported_frames = 0usize;
    let mut progress_collector = progress
        .map(|(callback, frame_interval)| {
            emit_progress(callback, "processing", 0, total_frames)?;
            Ok::<_, SubtrActorError>(CallbackCollector::with_frame_interval(
                |_frame, frame_number, _current_time| {
                    last_reported_frames = frame_number + 1;
                    emit_progress(callback, "processing", last_reported_frames, total_frames)
                },
                frame_interval.max(1),
            ))
        })
        .transpose()
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    let mut collectors: Vec<&mut dyn Collector> = vec![
        &mut replay_data_collector,
        &mut stats_collector,
        &mut boost_pad_collector,
    ];
    if let Some(progress_collector) = progress_collector.as_mut() {
        collectors.push(progress_collector);
    }

    processor
        .process_all(&mut collectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    if let Some((callback, _)) = progress {
        if last_reported_frames < total_frames {
            emit_progress(callback, "processing", total_frames, total_frames).map_err(|error| {
                JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
            })?;
        }
        emit_stage_progress(callback, "building-stats", 0.0)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let stats_timeline = stats_collector
        .into_replay_stats_timeline_scaffold()
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble stats timeline: {e:?}")))?;
    if let Some((callback, _)) = progress {
        emit_stage_progress(callback, "building-stats", 1.0)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let replay_data = replay_data_collector
        .into_replay_data_with_boost_pads(processor, boost_pad_collector.into_resolved_boost_pads())
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))?;
    if let Some((callback, _)) = progress {
        emit_stats_timeline_progress(callback, 0.35)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    Ok((replay_data, stats_timeline))
}

fn set_json_bytes<T: serde::Serialize>(
    object: &Object,
    key: &str,
    value: &T,
) -> Result<(), JsValue> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize {key}: {e}")))?;
    Reflect::set(
        object,
        &JsValue::from_str(key),
        &Uint8Array::from(bytes.as_slice()),
    )?;
    Ok(())
}

fn remove_event_derived_boost_fields(value: &mut Value) {
    let Some(boost) = value.as_object_mut() else {
        return;
    };
    for field in EVENT_DERIVED_BOOST_FIELDS {
        boost.remove(*field);
    }
}

fn remove_object_fields(value: &mut Value, fields: &[&str]) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    for field in fields {
        object.remove(*field);
    }
}

fn remove_empty_object_fields(value: &mut Value, fields: &[&str]) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    for field in fields {
        let is_empty = object
            .get(*field)
            .and_then(Value::as_object)
            .is_some_and(serde_json::Map::is_empty);
        if is_empty {
            object.remove(*field);
        }
    }
}

fn compact_stats_frame_for_transfer(
    frame: &subtr_actor::ReplayStatsFrame,
    compact_core: bool,
    compact_possession: bool,
    compact_pressure: bool,
    compact_movement: bool,
    compact_positioning: bool,
    compact_rotation: bool,
    compact_backboard: bool,
    compact_double_tap: bool,
    compact_ceiling_shot: bool,
    compact_one_timer: bool,
    compact_half_volley: bool,
    compact_pass: bool,
    compact_ball_carry: bool,
    compact_wall_aerial: bool,
    compact_wall_aerial_shot: bool,
    compact_flick: bool,
    compact_musty_flick: bool,
    compact_dodge_reset: bool,
    compact_powerslide: bool,
    compact_touch: bool,
    compact_rush: bool,
    compact_bump: bool,
    compact_fifty_fifty: bool,
    compact_demo: bool,
    compact_boost: bool,
    compact_speed_flip: bool,
    compact_half_flip: bool,
    compact_wavedash: bool,
    compact_whiff: bool,
) -> Result<Value, serde_json::Error> {
    let mut value = serde_json::to_value(frame)?;
    if compact_boost {
        if let Some(team_zero_boost) = value.pointer_mut("/team_zero/boost") {
            remove_event_derived_boost_fields(team_zero_boost);
        }
        if let Some(team_one_boost) = value.pointer_mut("/team_one/boost") {
            remove_event_derived_boost_fields(team_one_boost);
        }
    }
    if compact_core {
        if let Some(team_zero_core) = value.pointer_mut("/team_zero/core") {
            remove_object_fields(team_zero_core, EVENT_DERIVED_CORE_TEAM_FIELDS);
        }
        if let Some(team_one_core) = value.pointer_mut("/team_one/core") {
            remove_object_fields(team_one_core, EVENT_DERIVED_CORE_TEAM_FIELDS);
        }
    }
    if compact_possession {
        if let Some(team_zero_possession) = value.pointer_mut("/team_zero/possession") {
            remove_object_fields(team_zero_possession, EVENT_DERIVED_POSSESSION_TEAM_FIELDS);
        }
        if let Some(team_one_possession) = value.pointer_mut("/team_one/possession") {
            remove_object_fields(team_one_possession, EVENT_DERIVED_POSSESSION_TEAM_FIELDS);
        }
    }
    if compact_pressure {
        if let Some(team_zero_pressure) = value.pointer_mut("/team_zero/pressure") {
            remove_object_fields(team_zero_pressure, EVENT_DERIVED_PRESSURE_TEAM_FIELDS);
        }
        if let Some(team_one_pressure) = value.pointer_mut("/team_one/pressure") {
            remove_object_fields(team_one_pressure, EVENT_DERIVED_PRESSURE_TEAM_FIELDS);
        }
    }
    if compact_movement {
        if let Some(team_zero_movement) = value.pointer_mut("/team_zero/movement") {
            remove_object_fields(team_zero_movement, EVENT_DERIVED_MOVEMENT_FIELDS);
        }
        if let Some(team_one_movement) = value.pointer_mut("/team_one/movement") {
            remove_object_fields(team_one_movement, EVENT_DERIVED_MOVEMENT_FIELDS);
        }
    }
    if compact_rotation {
        if let Some(team_zero_rotation) = value.pointer_mut("/team_zero/rotation") {
            remove_object_fields(team_zero_rotation, EVENT_DERIVED_ROTATION_TEAM_FIELDS);
        }
        if let Some(team_one_rotation) = value.pointer_mut("/team_one/rotation") {
            remove_object_fields(team_one_rotation, EVENT_DERIVED_ROTATION_TEAM_FIELDS);
        }
    }
    if compact_backboard {
        if let Some(team_zero_backboard) = value.pointer_mut("/team_zero/backboard") {
            remove_object_fields(team_zero_backboard, EVENT_DERIVED_BACKBOARD_TEAM_FIELDS);
        }
        if let Some(team_one_backboard) = value.pointer_mut("/team_one/backboard") {
            remove_object_fields(team_one_backboard, EVENT_DERIVED_BACKBOARD_TEAM_FIELDS);
        }
    }
    if compact_double_tap {
        if let Some(team_zero_double_tap) = value.pointer_mut("/team_zero/double_tap") {
            remove_object_fields(team_zero_double_tap, EVENT_DERIVED_DOUBLE_TAP_TEAM_FIELDS);
        }
        if let Some(team_one_double_tap) = value.pointer_mut("/team_one/double_tap") {
            remove_object_fields(team_one_double_tap, EVENT_DERIVED_DOUBLE_TAP_TEAM_FIELDS);
        }
    }
    if compact_one_timer {
        if let Some(team_zero_one_timer) = value.pointer_mut("/team_zero/one_timer") {
            remove_object_fields(team_zero_one_timer, EVENT_DERIVED_ONE_TIMER_TEAM_FIELDS);
        }
        if let Some(team_one_one_timer) = value.pointer_mut("/team_one/one_timer") {
            remove_object_fields(team_one_one_timer, EVENT_DERIVED_ONE_TIMER_TEAM_FIELDS);
        }
    }
    if compact_half_volley {
        if let Some(team_zero_half_volley) = value.pointer_mut("/team_zero/half_volley") {
            remove_object_fields(team_zero_half_volley, EVENT_DERIVED_HALF_VOLLEY_TEAM_FIELDS);
        }
        if let Some(team_one_half_volley) = value.pointer_mut("/team_one/half_volley") {
            remove_object_fields(team_one_half_volley, EVENT_DERIVED_HALF_VOLLEY_TEAM_FIELDS);
        }
    }
    if compact_pass {
        if let Some(team_zero_pass) = value.pointer_mut("/team_zero/pass") {
            remove_object_fields(team_zero_pass, EVENT_DERIVED_PASS_TEAM_FIELDS);
        }
        if let Some(team_one_pass) = value.pointer_mut("/team_one/pass") {
            remove_object_fields(team_one_pass, EVENT_DERIVED_PASS_TEAM_FIELDS);
        }
    }
    if compact_ball_carry {
        if let Some(team_zero_ball_carry) = value.pointer_mut("/team_zero/ball_carry") {
            remove_object_fields(team_zero_ball_carry, EVENT_DERIVED_BALL_CARRY_FIELDS);
        }
        if let Some(team_one_ball_carry) = value.pointer_mut("/team_one/ball_carry") {
            remove_object_fields(team_one_ball_carry, EVENT_DERIVED_BALL_CARRY_FIELDS);
        }
        if let Some(team_zero_air_dribble) = value.pointer_mut("/team_zero/air_dribble") {
            remove_object_fields(team_zero_air_dribble, EVENT_DERIVED_AIR_DRIBBLE_FIELDS);
        }
        if let Some(team_one_air_dribble) = value.pointer_mut("/team_one/air_dribble") {
            remove_object_fields(team_one_air_dribble, EVENT_DERIVED_AIR_DRIBBLE_FIELDS);
        }
    }
    if compact_rush {
        if let Some(team_zero_rush) = value.pointer_mut("/team_zero/rush") {
            remove_object_fields(team_zero_rush, EVENT_DERIVED_RUSH_TEAM_FIELDS);
        }
        if let Some(team_one_rush) = value.pointer_mut("/team_one/rush") {
            remove_object_fields(team_one_rush, EVENT_DERIVED_RUSH_TEAM_FIELDS);
        }
    }
    if compact_bump {
        if let Some(team_zero_bump) = value.pointer_mut("/team_zero/bump") {
            remove_object_fields(team_zero_bump, EVENT_DERIVED_BUMP_TEAM_FIELDS);
        }
        if let Some(team_one_bump) = value.pointer_mut("/team_one/bump") {
            remove_object_fields(team_one_bump, EVENT_DERIVED_BUMP_TEAM_FIELDS);
        }
    }
    if compact_fifty_fifty {
        if let Some(team_zero_fifty_fifty) = value.pointer_mut("/team_zero/fifty_fifty") {
            remove_object_fields(team_zero_fifty_fifty, EVENT_DERIVED_FIFTY_FIFTY_TEAM_FIELDS);
        }
        if let Some(team_one_fifty_fifty) = value.pointer_mut("/team_one/fifty_fifty") {
            remove_object_fields(team_one_fifty_fifty, EVENT_DERIVED_FIFTY_FIFTY_TEAM_FIELDS);
        }
    }
    if compact_demo {
        if let Some(team_zero_demo) = value.pointer_mut("/team_zero/demo") {
            remove_object_fields(team_zero_demo, EVENT_DERIVED_DEMO_TEAM_FIELDS);
        }
        if let Some(team_one_demo) = value.pointer_mut("/team_one/demo") {
            remove_object_fields(team_one_demo, EVENT_DERIVED_DEMO_TEAM_FIELDS);
        }
    }
    if compact_powerslide {
        if let Some(team_zero_powerslide) = value.pointer_mut("/team_zero/powerslide") {
            remove_object_fields(team_zero_powerslide, EVENT_DERIVED_POWERSLIDE_FIELDS);
        }
        if let Some(team_one_powerslide) = value.pointer_mut("/team_one/powerslide") {
            remove_object_fields(team_one_powerslide, EVENT_DERIVED_POWERSLIDE_FIELDS);
        }
    }
    if let Some(players) = value.get_mut("players").and_then(Value::as_array_mut) {
        for player in players {
            if compact_core {
                if let Some(core) = player.get_mut("core") {
                    remove_object_fields(core, EVENT_DERIVED_CORE_PLAYER_FIELDS);
                }
            }
            if compact_movement {
                if let Some(movement) = player.get_mut("movement") {
                    remove_object_fields(movement, EVENT_DERIVED_MOVEMENT_FIELDS);
                }
            }
            if compact_positioning {
                if let Some(positioning) = player.get_mut("positioning") {
                    remove_object_fields(positioning, EVENT_DERIVED_POSITIONING_FIELDS);
                }
            }
            if compact_rotation {
                if let Some(rotation) = player.get_mut("rotation") {
                    remove_object_fields(rotation, EVENT_DERIVED_ROTATION_PLAYER_FIELDS);
                }
            }
            if compact_backboard {
                if let Some(backboard) = player.get_mut("backboard") {
                    remove_object_fields(backboard, EVENT_DERIVED_BACKBOARD_PLAYER_FIELDS);
                }
            }
            if compact_double_tap {
                if let Some(double_tap) = player.get_mut("double_tap") {
                    remove_object_fields(double_tap, EVENT_DERIVED_DOUBLE_TAP_PLAYER_FIELDS);
                }
            }
            if compact_ceiling_shot {
                if let Some(ceiling_shot) = player.get_mut("ceiling_shot") {
                    remove_object_fields(ceiling_shot, EVENT_DERIVED_CEILING_SHOT_FIELDS);
                }
            }
            if compact_one_timer {
                if let Some(one_timer) = player.get_mut("one_timer") {
                    remove_object_fields(one_timer, EVENT_DERIVED_ONE_TIMER_PLAYER_FIELDS);
                }
            }
            if compact_half_volley {
                if let Some(half_volley) = player.get_mut("half_volley") {
                    remove_object_fields(half_volley, EVENT_DERIVED_HALF_VOLLEY_PLAYER_FIELDS);
                }
            }
            if compact_pass {
                if let Some(pass) = player.get_mut("pass") {
                    remove_object_fields(pass, EVENT_DERIVED_PASS_PLAYER_FIELDS);
                }
            }
            if compact_ball_carry {
                if let Some(ball_carry) = player.get_mut("ball_carry") {
                    remove_object_fields(ball_carry, EVENT_DERIVED_BALL_CARRY_FIELDS);
                }
                if let Some(air_dribble) = player.get_mut("air_dribble") {
                    remove_object_fields(air_dribble, EVENT_DERIVED_AIR_DRIBBLE_FIELDS);
                }
            }
            if compact_wall_aerial {
                if let Some(wall_aerial) = player.get_mut("wall_aerial") {
                    remove_object_fields(wall_aerial, EVENT_DERIVED_WALL_AERIAL_FIELDS);
                }
            }
            if compact_wall_aerial_shot {
                if let Some(wall_aerial_shot) = player.get_mut("wall_aerial_shot") {
                    remove_object_fields(wall_aerial_shot, EVENT_DERIVED_WALL_AERIAL_SHOT_FIELDS);
                }
            }
            if compact_flick {
                if let Some(flick) = player.get_mut("flick") {
                    remove_object_fields(flick, EVENT_DERIVED_FLICK_FIELDS);
                }
            }
            if compact_musty_flick {
                if let Some(musty_flick) = player.get_mut("musty_flick") {
                    remove_object_fields(musty_flick, EVENT_DERIVED_MUSTY_FLICK_FIELDS);
                }
            }
            if compact_dodge_reset {
                if let Some(dodge_reset) = player.get_mut("dodge_reset") {
                    remove_object_fields(dodge_reset, EVENT_DERIVED_DODGE_RESET_FIELDS);
                }
            }
            if compact_powerslide {
                if let Some(powerslide) = player.get_mut("powerslide") {
                    remove_object_fields(powerslide, EVENT_DERIVED_POWERSLIDE_FIELDS);
                }
            }
            if compact_touch {
                if let Some(touch) = player.get_mut("touch") {
                    remove_object_fields(touch, EVENT_DERIVED_TOUCH_FIELDS);
                }
            }
            if compact_bump {
                if let Some(bump) = player.get_mut("bump") {
                    remove_object_fields(bump, EVENT_DERIVED_BUMP_PLAYER_FIELDS);
                }
            }
            if compact_fifty_fifty {
                if let Some(fifty_fifty) = player.get_mut("fifty_fifty") {
                    remove_object_fields(fifty_fifty, EVENT_DERIVED_FIFTY_FIFTY_PLAYER_FIELDS);
                }
            }
            if compact_demo {
                if let Some(demo) = player.get_mut("demo") {
                    remove_object_fields(demo, EVENT_DERIVED_DEMO_PLAYER_FIELDS);
                }
            }
            if compact_boost {
                if let Some(boost) = player.get_mut("boost") {
                    remove_event_derived_boost_fields(boost);
                }
            }
            if compact_speed_flip {
                if let Some(speed_flip) = player.get_mut("speed_flip") {
                    remove_object_fields(speed_flip, EVENT_DERIVED_SPEED_FLIP_FIELDS);
                }
            }
            if compact_half_flip {
                if let Some(half_flip) = player.get_mut("half_flip") {
                    remove_object_fields(half_flip, EVENT_DERIVED_HALF_FLIP_FIELDS);
                }
            }
            if compact_wavedash {
                if let Some(wavedash) = player.get_mut("wavedash") {
                    remove_object_fields(wavedash, EVENT_DERIVED_WAVEDASH_FIELDS);
                }
            }
            if compact_whiff {
                if let Some(whiff) = player.get_mut("whiff") {
                    remove_object_fields(whiff, EVENT_DERIVED_WHIFF_FIELDS);
                }
            }
            remove_empty_object_fields(player, PLAYER_STATS_MODULE_FIELDS);
        }
    }
    if let Some(team_zero) = value.pointer_mut("/team_zero") {
        remove_empty_object_fields(team_zero, TEAM_STATS_MODULE_FIELDS);
    }
    if let Some(team_one) = value.pointer_mut("/team_one") {
        remove_empty_object_fields(team_one, TEAM_STATS_MODULE_FIELDS);
    }
    Ok(value)
}

fn stats_timeline_json_parts(
    timeline: subtr_actor::ReplayStatsTimelineScaffold,
    max_frame_chunk_bytes: Option<usize>,
    progress: Option<(&Function, usize, f64, f64)>,
) -> Result<JsValue, JsValue> {
    let max_frame_chunk_bytes = max_frame_chunk_bytes
        .unwrap_or(DEFAULT_STATS_TIMELINE_FRAME_CHUNK_BYTES)
        .max(1024);
    let result = Object::new();
    set_json_bytes(&result, "config", &timeline.config)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.05))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    set_json_bytes(&result, "replayMeta", &timeline.replay_meta)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.1))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    set_json_bytes(&result, "events", &timeline.events)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.15))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let frame_chunks = Array::new();
    let mut current_chunk = Vec::new();
    current_chunk.push(b'[');
    let mut current_chunk_frames = 0usize;
    let total_frames = timeline.frames.len();

    for (frame_index, frame) in timeline.frames.iter().enumerate() {
        let frame_bytes = serde_json::to_vec(frame)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats frame: {e}")))?;
        let separator_bytes = usize::from(current_chunk_frames > 0);
        if current_chunk_frames > 0
            && current_chunk.len() + separator_bytes + frame_bytes.len() + 1 > max_frame_chunk_bytes
        {
            current_chunk.push(b']');
            frame_chunks.push(&Uint8Array::from(current_chunk.as_slice()));
            current_chunk = Vec::new();
            current_chunk.push(b'[');
            current_chunk_frames = 0;
        }
        if current_chunk_frames > 0 {
            current_chunk.push(b',');
        }
        current_chunk.extend_from_slice(&frame_bytes);
        current_chunk_frames += 1;

        if let Some((callback, report_every_n_frames, start, end)) = progress {
            let processed_frames = frame_index + 1;
            if processed_frames == total_frames
                || processed_frames.is_multiple_of(report_every_n_frames.max(1))
            {
                let frame_progress = if total_frames == 0 {
                    1.0
                } else {
                    processed_frames as f64 / total_frames as f64
                };
                let weighted_progress = start + ((end - start) * (0.15 + (frame_progress * 0.85)));
                emit_stats_timeline_progress(callback, weighted_progress).map_err(|error| {
                    JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
                })?;
            }
        }
    }

    current_chunk.push(b']');
    frame_chunks.push(&Uint8Array::from(current_chunk.as_slice()));
    Reflect::set(
        &result,
        &JsValue::from_str("frameChunks"),
        &frame_chunks.into(),
    )?;
    if let Some((callback, _, _, end)) = progress {
        emit_stats_timeline_progress(callback, end)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    Ok(result.into())
}

/// Parse a replay file and return the raw replay data as JavaScript object
#[wasm_bindgen]
pub fn parse_replay(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let replay_value = serde_json::to_value(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay: {e}")))?;

    serde_wasm_bindgen::to_value(&replay_value)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get NDArray data with metadata from replay data
#[wasm_bindgen]
pub fn get_ndarray_with_info(
    data: &[u8],
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
    fps: Option<f32>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let mut collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;

    // Use FrameRateDecorator with specified FPS (default 10.0)
    let mut decorated_collector =
        FrameRateDecorator::new_from_fps(fps.unwrap_or(10.0), &mut collector);

    let mut processor = ReplayProcessor::new(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to create processor: {e:?}")))?;

    processor
        .process(&mut decorated_collector)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    let (replay_meta_with_headers, ndarray) = collector
        .get_meta_and_ndarray()
        .map_err(|e| JsValue::from_str(&format!("Failed to get data: {e:?}")))?;

    // Convert ndarray to nested Vec for JavaScript
    let shape = ndarray.shape();
    let array_data: Vec<Vec<f32>> = ndarray.outer_iter().map(|row| row.to_vec()).collect();

    let result = serde_json::json!({
        "metadata": replay_meta_with_headers,
        "array_data": array_data,
        "shape": shape
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get only the replay metadata (without processing frames)
#[wasm_bindgen]
pub fn get_replay_meta(
    data: &[u8],
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let mut collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;

    let replay_meta = collector
        .process_and_get_meta_and_headers(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to get metadata: {e:?}")))?;

    serde_wasm_bindgen::to_value(&replay_meta)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get column headers for the NDArray (useful for understanding the data structure)
#[wasm_bindgen]
pub fn get_column_headers(
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<JsValue, JsValue> {
    let collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;
    let headers = collector.get_column_headers();

    serde_wasm_bindgen::to_value(&headers)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get structured frame data using ReplayDataCollector
/// This matches Python behavior - no FPS resampling, so goal frame numbers align
#[wasm_bindgen]
pub fn get_replay_frames_data(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let replay_data = collect_replay_data_with_optional_progress(&replay, None)?;

    serde_wasm_bindgen::to_value(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

#[wasm_bindgen]
pub fn get_replay_frames_data_with_progress(
    data: &[u8],
    callback: Function,
    report_every_n_frames: Option<usize>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let replay_data = collect_replay_data_with_optional_progress(
        &replay,
        Some((&callback, report_every_n_frames.unwrap_or(1000))),
    )?;

    serde_wasm_bindgen::to_value(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

#[wasm_bindgen]
pub fn get_replay_frames_data_json_with_progress(
    data: &[u8],
    callback: Function,
    report_every_n_frames: Option<usize>,
) -> Result<Vec<u8>, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let replay_data = collect_replay_data_with_optional_progress(
        &replay,
        Some((&callback, report_every_n_frames.unwrap_or(1000))),
    )?;
    serde_json::to_vec(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay data: {e}")))
}

#[wasm_bindgen]
pub fn get_replay_bundle_json_with_progress(
    data: &[u8],
    callback: Function,
    report_every_n_frames: Option<usize>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let (replay_data, stats_timeline) = collect_replay_bundle_with_optional_progress(
        &replay,
        Some((&callback, report_every_n_frames.unwrap_or(1000))),
    )?;

    let result = Object::new();
    {
        emit_stats_timeline_progress(&callback, 0.4)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
        let replay_data_bytes = serde_json::to_vec(&replay_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay data: {e}")))?;
        Reflect::set(
            &result,
            &JsValue::from_str("rawReplayData"),
            &Uint8Array::from(replay_data_bytes.as_slice()),
        )?;
        emit_stats_timeline_progress(&callback, 0.65)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    drop(replay_data);

    {
        emit_stats_timeline_progress(&callback, 0.75)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
        let stats_timeline_bytes = serde_json::to_vec(&stats_timeline)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats timeline: {e}")))?;
        Reflect::set(
            &result,
            &JsValue::from_str("statsTimeline"),
            &Uint8Array::from(stats_timeline_bytes.as_slice()),
        )?;
        emit_stats_timeline_progress(&callback, 1.0)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    Ok(result.into())
}

#[wasm_bindgen]
pub fn get_replay_bundle_json_parts_with_progress(
    data: &[u8],
    callback: Function,
    report_every_n_frames: Option<usize>,
    max_frame_chunk_bytes: Option<usize>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let (replay_data, stats_timeline) = collect_replay_bundle_with_optional_progress(
        &replay,
        Some((&callback, report_every_n_frames.unwrap_or(1000))),
    )?;

    let result = Object::new();
    {
        emit_stats_timeline_progress(&callback, 0.4)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
        let replay_data_bytes = serde_json::to_vec(&replay_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay data: {e}")))?;
        Reflect::set(
            &result,
            &JsValue::from_str("rawReplayData"),
            &Uint8Array::from(replay_data_bytes.as_slice()),
        )?;
        emit_stats_timeline_progress(&callback, 0.55)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    drop(replay_data);

    let report_every_n_frames = report_every_n_frames.unwrap_or(1000);
    Reflect::set(
        &result,
        &JsValue::from_str("statsTimelineParts"),
        &stats_timeline_json_parts(
            stats_timeline,
            max_frame_chunk_bytes,
            Some((&callback, report_every_n_frames, 0.6, 0.9)),
        )?,
    )?;
    emit_stats_timeline_progress(&callback, 0.92)
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    Ok(result.into())
}

/// Get cumulative stats snapshots for each replay sample.
#[wasm_bindgen]
pub fn get_stats_timeline(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let stats_timeline = StatsCollector::new()
        .get_replay_stats_timeline(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay stats: {e:?}")))?;

    serde_wasm_bindgen::to_value(&stats_timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

#[wasm_bindgen]
pub fn get_stats_timeline_json(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let stats_timeline = StatsCollector::new()
        .get_replay_stats_timeline(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay stats: {e:?}")))?;

    serde_json::to_vec(&stats_timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats timeline: {e}")))
}

#[wasm_bindgen]
pub fn get_stats_timeline_json_parts(
    data: &[u8],
    max_frame_chunk_bytes: Option<usize>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let stats_timeline = StatsTimelineEventCollector::new()
        .get_replay_data(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay stats: {e:?}")))?;

    stats_timeline_json_parts(stats_timeline, max_frame_chunk_bytes, None)
}

/// Validate that a replay file can be parsed
#[wasm_bindgen]
pub fn validate_replay(data: &[u8]) -> Result<JsValue, JsValue> {
    match parse_replay_from_data(data) {
        Ok(_) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": true,
            "message": "Replay is valid"
        })),
        Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": false,
            "error": e.as_string().unwrap_or_else(|| "Unknown error".to_string())
        })),
    }
    .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get basic replay information (version, etc.)
#[wasm_bindgen]
pub fn get_replay_info(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let info = serde_json::json!({
        "header_size": replay.header_size,
        "major_version": replay.major_version,
        "minor_version": replay.minor_version,
        "net_version": replay.net_version,
        "properties_count": replay.properties.len()
    });

    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

fn build_ndarray_collector(
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<NDArrayCollector<f32>, JsValue> {
    let global_feature_adders = global_feature_adders.unwrap_or_else(|| {
        DEFAULT_GLOBAL_FEATURE_ADDERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    });
    let player_feature_adders = player_feature_adders.unwrap_or_else(|| {
        DEFAULT_PLAYER_FEATURE_ADDERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    });

    let global_strs: Vec<&str> = global_feature_adders.iter().map(|s| s.as_str()).collect();
    let player_strs: Vec<&str> = player_feature_adders.iter().map(|s| s.as_str()).collect();

    NDArrayCollector::<f32>::from_strings(&global_strs, &player_strs)
        .map_err(|e| JsValue::from_str(&format!("Failed to build collector: {e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxcars::RemoteId;
    use subtr_actor::{
        AirDribbleStats, BackboardPlayerStats, BackboardTeamStats, BallCarryStats, BoostStats,
        BumpPlayerStats, BumpTeamStats, CeilingShotStats, CorePlayerStats, CoreTeamStats,
        DemoPlayerStats, DemoTeamStats, DodgeResetStats, DoubleTapPlayerStats, DoubleTapTeamStats,
        FiftyFiftyPlayerStats, FiftyFiftyTeamStats, FlickStats, GameplayPhase, HalfFlipStats,
        HalfVolleyPlayerStats, HalfVolleyTeamStats, MovementStats, MustyFlickStats,
        OneTimerPlayerStats, OneTimerTeamStats, PassPlayerStats, PassTeamStats,
        PlayerStatsSnapshot, PositioningStats, PossessionTeamStats, PowerslideStats,
        PressureTeamStats, ReplayStatsFrame, RotationPlayerStats, RotationTeamStats, RushTeamStats,
        SpeedFlipStats, TeamStatsSnapshot, TouchStats, WallAerialShotStats, WallAerialStats,
        WavedashStats, WhiffStats,
    };

    fn default_team_stats_snapshot() -> TeamStatsSnapshot {
        TeamStatsSnapshot {
            fifty_fifty: FiftyFiftyTeamStats::default(),
            possession: PossessionTeamStats::default(),
            pressure: PressureTeamStats::default(),
            rotation: RotationTeamStats::default(),
            rush: RushTeamStats::default(),
            core: CoreTeamStats::default(),
            backboard: BackboardTeamStats::default(),
            double_tap: DoubleTapTeamStats::default(),
            one_timer: OneTimerTeamStats::default(),
            pass: PassTeamStats::default(),
            ball_carry: BallCarryStats::default(),
            air_dribble: AirDribbleStats::default(),
            boost: BoostStats::default(),
            bump: BumpTeamStats::default(),
            half_volley: HalfVolleyTeamStats::default(),
            movement: MovementStats::default(),
            powerslide: PowerslideStats::default(),
            demo: DemoTeamStats::default(),
        }
    }

    fn default_player_stats_snapshot() -> PlayerStatsSnapshot {
        PlayerStatsSnapshot {
            player_id: RemoteId::Steam(1),
            name: "Blue".to_owned(),
            is_team_0: true,
            core: CorePlayerStats::default(),
            backboard: BackboardPlayerStats::default(),
            ceiling_shot: CeilingShotStats::default(),
            wall_aerial: WallAerialStats::default(),
            wall_aerial_shot: WallAerialShotStats::default(),
            double_tap: DoubleTapPlayerStats::default(),
            one_timer: OneTimerPlayerStats::default(),
            pass: PassPlayerStats::default(),
            fifty_fifty: FiftyFiftyPlayerStats::default(),
            speed_flip: SpeedFlipStats::default(),
            half_flip: HalfFlipStats::default(),
            half_volley: HalfVolleyPlayerStats::default(),
            wavedash: WavedashStats::default(),
            touch: TouchStats::default(),
            whiff: WhiffStats::default(),
            flick: FlickStats::default(),
            musty_flick: MustyFlickStats::default(),
            dodge_reset: DodgeResetStats::default(),
            ball_carry: BallCarryStats::default(),
            air_dribble: AirDribbleStats::default(),
            boost: BoostStats::default(),
            bump: BumpPlayerStats::default(),
            movement: MovementStats::default(),
            positioning: PositioningStats::default(),
            rotation: RotationPlayerStats::default(),
            powerslide: PowerslideStats::default(),
            demo: DemoPlayerStats::default(),
        }
    }

    fn default_stats_frame() -> ReplayStatsFrame {
        ReplayStatsFrame {
            frame_number: 10,
            time: 1.0,
            dt: 1.0 / 30.0,
            seconds_remaining: Some(300),
            game_state: Some(0),
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
            team_zero: default_team_stats_snapshot(),
            team_one: default_team_stats_snapshot(),
            players: vec![default_player_stats_snapshot()],
        }
    }

    fn assert_missing_fields(value: &Value, pointer: &str, fields: &[&str]) {
        let Some(object) = value.pointer(pointer).and_then(Value::as_object) else {
            return;
        };
        for field in fields {
            assert!(
                !object.contains_key(*field),
                "{pointer} should not serialize event-derived field {field}"
            );
        }
    }

    fn assert_empty_object(value: &Value, pointer: &str) {
        let object = value
            .pointer(pointer)
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{pointer} should be an object"));
        assert!(
            object.is_empty(),
            "{pointer} should not serialize any per-frame stat fields, found {:?}",
            object.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn transfer_compaction_removes_event_derived_partial_sums() {
        let compacted = compact_stats_frame_for_transfer(
            &default_stats_frame(),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("frame should compact");

        assert_eq!(compacted.pointer("/frame_number"), Some(&Value::from(10)));
        assert_eq!(
            compacted.pointer("/players/0/name"),
            Some(&Value::from("Blue"))
        );

        assert_missing_fields(
            &compacted,
            "/team_zero/core",
            EVENT_DERIVED_CORE_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/possession",
            EVENT_DERIVED_POSSESSION_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/pressure",
            EVENT_DERIVED_PRESSURE_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/movement",
            EVENT_DERIVED_MOVEMENT_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/rotation",
            EVENT_DERIVED_ROTATION_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/backboard",
            EVENT_DERIVED_BACKBOARD_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/double_tap",
            EVENT_DERIVED_DOUBLE_TAP_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/one_timer",
            EVENT_DERIVED_ONE_TIMER_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/half_volley",
            EVENT_DERIVED_HALF_VOLLEY_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/pass",
            EVENT_DERIVED_PASS_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/ball_carry",
            EVENT_DERIVED_BALL_CARRY_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/air_dribble",
            EVENT_DERIVED_AIR_DRIBBLE_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/rush",
            EVENT_DERIVED_RUSH_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/bump",
            EVENT_DERIVED_BUMP_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/fifty_fifty",
            EVENT_DERIVED_FIFTY_FIFTY_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/demo",
            EVENT_DERIVED_DEMO_TEAM_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/team_zero/powerslide",
            EVENT_DERIVED_POWERSLIDE_FIELDS,
        );
        assert_missing_fields(&compacted, "/team_zero/boost", EVENT_DERIVED_BOOST_FIELDS);

        assert_missing_fields(
            &compacted,
            "/players/0/core",
            EVENT_DERIVED_CORE_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/movement",
            EVENT_DERIVED_MOVEMENT_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/positioning",
            EVENT_DERIVED_POSITIONING_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/rotation",
            EVENT_DERIVED_ROTATION_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/backboard",
            EVENT_DERIVED_BACKBOARD_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/double_tap",
            EVENT_DERIVED_DOUBLE_TAP_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/ceiling_shot",
            EVENT_DERIVED_CEILING_SHOT_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/one_timer",
            EVENT_DERIVED_ONE_TIMER_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/half_volley",
            EVENT_DERIVED_HALF_VOLLEY_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/pass",
            EVENT_DERIVED_PASS_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/ball_carry",
            EVENT_DERIVED_BALL_CARRY_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/air_dribble",
            EVENT_DERIVED_AIR_DRIBBLE_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/wall_aerial",
            EVENT_DERIVED_WALL_AERIAL_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/wall_aerial_shot",
            EVENT_DERIVED_WALL_AERIAL_SHOT_FIELDS,
        );
        assert_missing_fields(&compacted, "/players/0/flick", EVENT_DERIVED_FLICK_FIELDS);
        assert_missing_fields(
            &compacted,
            "/players/0/musty_flick",
            EVENT_DERIVED_MUSTY_FLICK_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/dodge_reset",
            EVENT_DERIVED_DODGE_RESET_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/powerslide",
            EVENT_DERIVED_POWERSLIDE_FIELDS,
        );
        assert_missing_fields(&compacted, "/players/0/touch", EVENT_DERIVED_TOUCH_FIELDS);
        assert_missing_fields(
            &compacted,
            "/players/0/bump",
            EVENT_DERIVED_BUMP_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/fifty_fifty",
            EVENT_DERIVED_FIFTY_FIFTY_PLAYER_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/demo",
            EVENT_DERIVED_DEMO_PLAYER_FIELDS,
        );
        assert_missing_fields(&compacted, "/players/0/boost", EVENT_DERIVED_BOOST_FIELDS);
        assert_missing_fields(
            &compacted,
            "/players/0/speed_flip",
            EVENT_DERIVED_SPEED_FLIP_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/half_flip",
            EVENT_DERIVED_HALF_FLIP_FIELDS,
        );
        assert_missing_fields(
            &compacted,
            "/players/0/wavedash",
            EVENT_DERIVED_WAVEDASH_FIELDS,
        );
        assert_missing_fields(&compacted, "/players/0/whiff", EVENT_DERIVED_WHIFF_FIELDS);

        assert_empty_object(&compacted, "/team_zero");
        assert_empty_object(&compacted, "/team_one");

        let player_modules = [
            "core",
            "backboard",
            "ceiling_shot",
            "wall_aerial",
            "wall_aerial_shot",
            "double_tap",
            "one_timer",
            "pass",
            "fifty_fifty",
            "speed_flip",
            "half_flip",
            "half_volley",
            "wavedash",
            "touch",
            "whiff",
            "flick",
            "musty_flick",
            "dodge_reset",
            "ball_carry",
            "air_dribble",
            "boost",
            "bump",
            "movement",
            "positioning",
            "rotation",
            "powerslide",
            "demo",
        ];
        let player = compacted
            .pointer("/players/0")
            .and_then(Value::as_object)
            .expect("player should be an object");
        let identity_fields = ["is_team_0", "name", "player_id"];
        assert_eq!(
            player.len(),
            identity_fields.len(),
            "compacted players should only serialize identity fields"
        );
        for field in identity_fields {
            assert!(
                player.contains_key(field),
                "compacted player should retain identity field {field}"
            );
        }
        for module in player_modules {
            assert!(
                !player.contains_key(module),
                "compacted player should not serialize stat module {module}"
            );
        }
    }
}
