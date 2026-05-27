use super::playback_config_helpers::*;
use super::playback_config_timeline_goals_extra::{
    apply_empty_net_goal_timeline_config, apply_event_goal_timeline_config,
};
use super::*;

pub(super) fn apply_goal_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    output.aerial_goal_min_ball_z = goal_f32(
        config,
        "aerial_goal",
        "aerial_goal_min_ball_z",
        output.aerial_goal_min_ball_z,
    );
    output.high_aerial_goal_min_ball_z = goal_f32(
        config,
        "high_aerial_goal",
        "high_aerial_goal_min_ball_z",
        output.high_aerial_goal_min_ball_z,
    );
    output.long_distance_goal_max_attacking_y = goal_f32(
        config,
        "long_distance_goal",
        "long_distance_goal_max_attacking_y",
        output.long_distance_goal_max_attacking_y,
    );
    output.own_half_goal_max_attacking_y = goal_f32(
        config,
        "own_half_goal",
        "own_half_goal_max_attacking_y",
        output.own_half_goal_max_attacking_y,
    );
    apply_empty_net_goal_timeline_config(output, config);
    apply_event_goal_timeline_config(output, config);
    output.half_volley_max_bounce_to_touch_seconds = goal_f32(
        config,
        "half_volley",
        "half_volley_max_bounce_to_touch_seconds",
        output.half_volley_max_bounce_to_touch_seconds,
    );
    output.half_volley_min_ball_speed = goal_f32(
        config,
        "half_volley",
        "half_volley_min_ball_speed",
        output.half_volley_min_ball_speed,
    );
    output.half_volley_goal_max_touch_to_goal_seconds = goal_f32(
        config,
        "half_volley_goal",
        "half_volley_goal_max_touch_to_goal_seconds",
        output.half_volley_goal_max_touch_to_goal_seconds,
    );
    output.half_volley_goal_min_goal_alignment = goal_f32(
        config,
        "half_volley_goal",
        "half_volley_goal_min_goal_alignment",
        output.half_volley_goal_min_goal_alignment,
    );
}

pub(super) fn goal_f32(config: &Map<String, Value>, module: &str, key: &str, default: f32) -> f32 {
    f32_config(module_config(config, module), key, default)
}

pub(super) fn event_goal_f32(
    config: &Map<String, Value>,
    module: &str,
    primary_key: &str,
    fallback_key: &str,
    default: f32,
) -> f32 {
    json_config_f32(module_config(config, module), primary_key, fallback_key).unwrap_or(default)
}
