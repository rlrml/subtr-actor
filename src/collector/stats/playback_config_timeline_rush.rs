use super::playback_config_helpers::*;
use super::*;

pub(super) fn apply_rush_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    let rush = module_config(config, "rush");
    output.rush_max_start_y = f32_config(rush, "rush_max_start_y", output.rush_max_start_y);
    output.rush_attack_support_distance_y = f32_config(
        rush,
        "rush_attack_support_distance_y",
        output.rush_attack_support_distance_y,
    );
    output.rush_defender_distance_y = f32_config(
        rush,
        "rush_defender_distance_y",
        output.rush_defender_distance_y,
    );
    output.rush_min_possession_retained_seconds = f32_config(
        rush,
        "rush_min_possession_retained_seconds",
        output.rush_min_possession_retained_seconds,
    );
}
