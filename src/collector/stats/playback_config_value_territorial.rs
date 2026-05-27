use super::playback_config_helpers::*;
use super::*;

pub(super) fn insert_territorial_pressure_config_values(
    output: &mut Map<String, Value>,
    territorial: JsonObject<'_>,
) -> SubtrActorResult<()> {
    let defaults = TerritorialPressureCalculatorConfig::default();
    insert_config_pairs(
        output,
        &[
            f64_config_with_source_key(
                territorial,
                "territorial_pressure_neutral_zone_half_width_y",
                "territorial_pressure_neutral_zone_half_width_y",
                defaults.neutral_zone_half_width_y,
            ),
            f64_config_with_source_key(
                territorial,
                "territorial_pressure_min_establish_seconds",
                "territorial_pressure_min_establish_seconds",
                defaults.min_establish_seconds,
            ),
            f64_config_with_source_key(
                territorial,
                "territorial_pressure_min_establish_third_seconds",
                "territorial_pressure_min_establish_third_seconds",
                defaults.min_establish_third_seconds,
            ),
            f64_config_with_source_key(
                territorial,
                "territorial_pressure_relief_grace_seconds",
                "territorial_pressure_relief_grace_seconds",
                defaults.relief_grace_seconds,
            ),
            f64_config_with_source_key(
                territorial,
                "territorial_pressure_confirmed_relief_grace_seconds",
                "territorial_pressure_confirmed_relief_grace_seconds",
                defaults.confirmed_relief_grace_seconds,
            ),
        ],
    )
}
