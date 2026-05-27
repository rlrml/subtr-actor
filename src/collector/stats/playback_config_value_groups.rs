use super::playback_config_helpers::*;
use super::*;

pub(super) fn insert_positioning_config_values(
    output: &mut Map<String, Value>,
    positioning: JsonObject<'_>,
) -> SubtrActorResult<()> {
    let defaults = PositioningCalculatorConfig::default();
    insert_config_pairs(
        output,
        &[
            (
                "most_back_forward_threshold_y",
                f64_config(
                    positioning,
                    "most_back_forward_threshold_y",
                    defaults.most_back_forward_threshold_y,
                ),
            ),
            (
                "level_ball_depth_margin",
                f64_config(
                    positioning,
                    "level_ball_depth_margin",
                    defaults.level_ball_depth_margin,
                ),
            ),
        ],
    )
}

pub(super) fn insert_pressure_config_values(
    output: &mut Map<String, Value>,
    pressure: JsonObject<'_>,
) -> SubtrActorResult<()> {
    insert_f64_config(
        output,
        "pressure_neutral_zone_half_width_y",
        f64_config(
            pressure,
            "pressure_neutral_zone_half_width_y",
            PressureCalculatorConfig::default().neutral_zone_half_width_y,
        ),
    )
}

pub(super) fn insert_rotation_config_values(
    output: &mut Map<String, Value>,
    rotation: JsonObject<'_>,
) -> SubtrActorResult<()> {
    let defaults = RotationCalculatorConfig::default();
    insert_config_pairs(
        output,
        &[
            f64_config_with_source_key(
                rotation,
                "rotation_role_depth_margin",
                "role_depth_margin",
                defaults.role_depth_margin,
            ),
            f64_config_with_source_key(
                rotation,
                "rotation_first_man_ambiguity_margin",
                "first_man_ambiguity_margin",
                defaults.first_man_ambiguity_margin,
            ),
            f64_config_with_source_key(
                rotation,
                "rotation_first_man_debounce_seconds",
                "first_man_debounce_seconds",
                defaults.first_man_debounce_seconds,
            ),
        ],
    )
}
