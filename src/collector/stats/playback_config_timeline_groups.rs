use super::playback_config_helpers::*;
use super::*;

pub(super) fn apply_positioning_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    let positioning = module_config(config, "positioning");
    output.most_back_forward_threshold_y = f32_config(
        positioning,
        "most_back_forward_threshold_y",
        output.most_back_forward_threshold_y,
    );
    output.level_ball_depth_margin = f32_config(
        positioning,
        "level_ball_depth_margin",
        output.level_ball_depth_margin,
    );
}

pub(super) fn apply_pressure_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    output.pressure_neutral_zone_half_width_y = f32_config(
        module_config(config, "pressure"),
        "pressure_neutral_zone_half_width_y",
        output.pressure_neutral_zone_half_width_y,
    );
}

pub(super) fn apply_territorial_pressure_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    let territorial = module_config(config, "territorial_pressure");
    output.territorial_pressure_neutral_zone_half_width_y = f32_config(
        territorial,
        "territorial_pressure_neutral_zone_half_width_y",
        output.territorial_pressure_neutral_zone_half_width_y,
    );
    output.territorial_pressure_min_establish_seconds = f32_config(
        territorial,
        "territorial_pressure_min_establish_seconds",
        output.territorial_pressure_min_establish_seconds,
    );
    output.territorial_pressure_min_establish_third_seconds = f32_config(
        territorial,
        "territorial_pressure_min_establish_third_seconds",
        output.territorial_pressure_min_establish_third_seconds,
    );
    output.territorial_pressure_relief_grace_seconds = f32_config(
        territorial,
        "territorial_pressure_relief_grace_seconds",
        output.territorial_pressure_relief_grace_seconds,
    );
    output.territorial_pressure_confirmed_relief_grace_seconds = f32_config(
        territorial,
        "territorial_pressure_confirmed_relief_grace_seconds",
        output.territorial_pressure_confirmed_relief_grace_seconds,
    );
}

pub(super) fn apply_rotation_timeline_config(
    output: &mut StatsTimelineConfig,
    config: &Map<String, Value>,
) {
    let rotation = module_config(config, "rotation");
    output.rotation_role_depth_margin = f32_config(
        rotation,
        "role_depth_margin",
        output.rotation_role_depth_margin,
    );
    output.rotation_first_man_ambiguity_margin = f32_config(
        rotation,
        "first_man_ambiguity_margin",
        output.rotation_first_man_ambiguity_margin,
    );
    output.rotation_first_man_debounce_seconds = f32_config(
        rotation,
        "first_man_debounce_seconds",
        output.rotation_first_man_debounce_seconds,
    );
}
