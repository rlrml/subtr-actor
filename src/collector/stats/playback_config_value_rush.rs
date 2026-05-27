use super::playback_config_helpers::*;
use super::*;

pub(super) fn insert_rush_config_values(
    output: &mut Map<String, Value>,
    rush: JsonObject<'_>,
) -> SubtrActorResult<()> {
    let defaults = RushCalculatorConfig::default();
    insert_config_pairs(
        output,
        &[
            f64_config_with_source_key(
                rush,
                "rush_max_start_y",
                "rush_max_start_y",
                defaults.max_start_y,
            ),
            f64_config_with_source_key(
                rush,
                "rush_attack_support_distance_y",
                "rush_attack_support_distance_y",
                defaults.attack_support_distance_y,
            ),
            f64_config_with_source_key(
                rush,
                "rush_defender_distance_y",
                "rush_defender_distance_y",
                defaults.defender_distance_y,
            ),
            f64_config_with_source_key(
                rush,
                "rush_min_possession_retained_seconds",
                "rush_min_possession_retained_seconds",
                defaults.min_possession_retained_seconds,
            ),
        ],
    )
}
