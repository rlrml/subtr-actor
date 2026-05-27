use super::playback_config_timeline_goals::{event_goal_f32, goal_f32};
use super::*;

pub(super) fn apply_empty_net_goal_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    output.empty_net_min_defender_y_margin = goal_f32(
        config,
        "empty_net_goal",
        "empty_net_min_defender_y_margin",
        output.empty_net_min_defender_y_margin,
    );
    output.empty_net_min_defender_distance = goal_f32(
        config,
        "empty_net_goal",
        "empty_net_min_defender_distance",
        output.empty_net_min_defender_distance,
    );
    output.empty_net_max_touch_attacking_y = goal_f32(
        config,
        "empty_net_goal",
        "empty_net_max_touch_attacking_y",
        output.empty_net_max_touch_attacking_y,
    );
}

pub(super) fn apply_event_goal_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    output.flick_goal_max_event_to_goal_seconds = event_goal_f32(
        config,
        "flick_goal",
        "flick_goal_max_event_to_goal_seconds",
        "flick_goal_max_event_to_touch_seconds",
        output.flick_goal_max_event_to_goal_seconds,
    );
    output.double_tap_goal_max_event_to_goal_seconds = event_goal_f32(
        config,
        "double_tap_goal",
        "double_tap_goal_max_event_to_goal_seconds",
        "double_tap_goal_max_event_to_touch_seconds",
        output.double_tap_goal_max_event_to_goal_seconds,
    );
    output.one_timer_goal_max_event_to_goal_seconds = event_goal_f32(
        config,
        "one_timer_goal",
        "one_timer_goal_max_event_to_goal_seconds",
        "one_timer_goal_max_event_to_touch_seconds",
        output.one_timer_goal_max_event_to_goal_seconds,
    );
    output.air_dribble_goal_max_end_to_goal_seconds = event_goal_f32(
        config,
        "air_dribble_goal",
        "air_dribble_goal_max_end_to_goal_seconds",
        "air_dribble_goal_max_end_to_touch_seconds",
        output.air_dribble_goal_max_end_to_goal_seconds,
    );
    output.flip_reset_goal_max_event_to_goal_seconds = event_goal_f32(
        config,
        "flip_reset_goal",
        "flip_reset_goal_max_event_to_goal_seconds",
        "flip_reset_goal_max_event_to_touch_seconds",
        output.flip_reset_goal_max_event_to_goal_seconds,
    );
}
