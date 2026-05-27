use crate::*;

pub fn default_stats_timeline_config() -> StatsTimelineConfig {
    let rotation_defaults = RotationCalculatorConfig::default();
    let territorial_pressure_defaults = TerritorialPressureCalculatorConfig::default();
    StatsTimelineConfig {
        most_back_forward_threshold_y: PositioningCalculatorConfig::default()
            .most_back_forward_threshold_y,
        level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
        pressure_neutral_zone_half_width_y: PressureCalculatorConfig::default()
            .neutral_zone_half_width_y,
        territorial_pressure_neutral_zone_half_width_y: territorial_pressure_defaults
            .neutral_zone_half_width_y,
        territorial_pressure_min_establish_seconds: territorial_pressure_defaults
            .min_establish_seconds,
        territorial_pressure_min_establish_third_seconds: territorial_pressure_defaults
            .min_establish_third_seconds,
        territorial_pressure_relief_grace_seconds: territorial_pressure_defaults
            .relief_grace_seconds,
        territorial_pressure_confirmed_relief_grace_seconds: territorial_pressure_defaults
            .confirmed_relief_grace_seconds,
        rotation_role_depth_margin: rotation_defaults.role_depth_margin,
        rotation_first_man_ambiguity_margin: rotation_defaults.first_man_ambiguity_margin,
        rotation_first_man_debounce_seconds: rotation_defaults.first_man_debounce_seconds,
        rush_max_start_y: RushCalculatorConfig::default().max_start_y,
        rush_attack_support_distance_y: RushCalculatorConfig::default().attack_support_distance_y,
        rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
        rush_min_possession_retained_seconds: RushCalculatorConfig::default()
            .min_possession_retained_seconds,
        aerial_goal_min_ball_z: AerialGoalCalculatorConfig::default().min_ball_z,
        high_aerial_goal_min_ball_z: HighAerialGoalCalculatorConfig::default().min_ball_z,
        long_distance_goal_max_attacking_y: LongDistanceGoalCalculatorConfig::default()
            .max_attacking_y,
        own_half_goal_max_attacking_y: OwnHalfGoalCalculatorConfig::default().max_attacking_y,
        empty_net_min_defender_y_margin: EmptyNetGoalCalculatorConfig::default()
            .min_defender_y_margin,
        empty_net_min_defender_distance: EmptyNetGoalCalculatorConfig::default()
            .min_defender_distance,
        empty_net_max_touch_attacking_y: EmptyNetGoalCalculatorConfig::default()
            .max_touch_attacking_y,
        flick_goal_max_event_to_goal_seconds: FlickGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        double_tap_goal_max_event_to_goal_seconds: DoubleTapGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        one_timer_goal_max_event_to_goal_seconds: OneTimerGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        air_dribble_goal_max_end_to_goal_seconds: AirDribbleGoalCalculatorConfig::default()
            .max_end_to_goal_seconds,
        flip_reset_goal_max_event_to_goal_seconds: FlipResetGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        half_volley_max_bounce_to_touch_seconds: HalfVolleyCalculatorConfig::default()
            .max_bounce_to_touch_seconds,
        half_volley_min_ball_speed: HalfVolleyCalculatorConfig::default().min_ball_speed,
        half_volley_goal_max_touch_to_goal_seconds: HalfVolleyGoalCalculatorConfig::default()
            .max_touch_to_goal_seconds,
        half_volley_goal_min_goal_alignment: HalfVolleyGoalCalculatorConfig::default()
            .min_goal_alignment,
    }
}
