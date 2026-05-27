use super::playback_config_helpers::*;
use super::playback_config_value_goals_extra::extra_goal_values;
use super::*;

pub(super) fn insert_goal_config_values(
    output: &mut Map<String, Value>,
    config: &Map<String, Value>,
) -> SubtrActorResult<()> {
    let mut values = vec![
        goal_value(
            config,
            "aerial_goal",
            "aerial_goal_min_ball_z",
            AerialGoalCalculatorConfig::default().min_ball_z,
        ),
        goal_value(
            config,
            "high_aerial_goal",
            "high_aerial_goal_min_ball_z",
            HighAerialGoalCalculatorConfig::default().min_ball_z,
        ),
        goal_value(
            config,
            "long_distance_goal",
            "long_distance_goal_max_attacking_y",
            LongDistanceGoalCalculatorConfig::default().max_attacking_y,
        ),
        goal_value(
            config,
            "own_half_goal",
            "own_half_goal_max_attacking_y",
            OwnHalfGoalCalculatorConfig::default().max_attacking_y,
        ),
        goal_value(
            config,
            "empty_net_goal",
            "empty_net_min_defender_y_margin",
            EmptyNetGoalCalculatorConfig::default().min_defender_y_margin,
        ),
        goal_value(
            config,
            "empty_net_goal",
            "empty_net_min_defender_distance",
            EmptyNetGoalCalculatorConfig::default().min_defender_distance,
        ),
        goal_value(
            config,
            "empty_net_goal",
            "empty_net_max_touch_attacking_y",
            EmptyNetGoalCalculatorConfig::default().max_touch_attacking_y,
        ),
    ];
    values.extend(extra_goal_values(config));
    insert_config_pairs(output, &values)
}

pub(super) fn goal_value(
    config: &Map<String, Value>,
    module: &str,
    key: &'static str,
    default: f32,
) -> (&'static str, f64) {
    (key, f64_config(module_config(config, module), key, default))
}
