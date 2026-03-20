use subtr_actor::*;

fn find_field<'a>(fields: &'a [ExportedStat], domain: &str, name: &str) -> &'a ExportedStat {
    fields
        .iter()
        .find(|field| field.descriptor.domain == domain && field.descriptor.name == name)
        .unwrap_or_else(|| panic!("Missing field {domain}.{name}"))
}

#[test]
fn test_positioning_stats_export_includes_derived_metrics() {
    let stats = PositioningStats {
        active_game_time: 12.0,
        tracked_time: 10.0,
        sum_distance_to_teammates: 2_500.0,
        sum_distance_to_ball: 20_000.0,
        sum_distance_to_ball_has_possession: 4_000.0,
        time_has_possession: 2.0,
        sum_distance_to_ball_no_possession: 15_000.0,
        time_no_possession: 5.0,
        time_demolished: 1.0,
        time_no_teammates: 2.0,
        time_most_back: 3.0,
        time_most_forward: 1.0,
        time_mid_role: 1.0,
        time_other_role: 2.0,
        time_defensive_zone: 4.0,
        time_neutral_zone: 3.0,
        time_offensive_zone: 3.0,
        time_defensive_half: 6.0,
        time_offensive_half: 4.0,
        time_closest_to_ball: 2.0,
        time_farthest_from_ball: 4.0,
        time_behind_ball: 7.0,
        time_in_front_of_ball: 3.0,
        times_caught_ahead_of_play_on_conceded_goals: 0,
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "positioning", "avg_distance_to_ball").value,
        StatValue::Float(2_000.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "avg_distance_to_ball_possession").value,
        StatValue::Float(2_000.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "avg_distance_to_mates").value,
        StatValue::Float(250.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "percent_behind_ball").value,
        StatValue::Float(70.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "time_defensive_third").value,
        StatValue::Float(4.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "active_game_time").value,
        StatValue::Float(12.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "time_demolished").value,
        StatValue::Float(1.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "time_no_teammates").value,
        StatValue::Float(2.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "time_mid_role").value,
        StatValue::Float(1.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "time_other_role").value,
        StatValue::Float(2.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "percent_neutral_third").value,
        StatValue::Float(30.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "percent_most_back").value,
        StatValue::Float(30.0)
    );
}

#[test]
fn test_core_player_stats_export_uses_legacy_variant_metadata() {
    let mut goal_after_kickoff = GoalAfterKickoffStats::default();
    goal_after_kickoff.record_goal(6.0);
    goal_after_kickoff.record_goal(15.0);

    let stats = CorePlayerStats {
        score: 500,
        goals: 2,
        assists: 1,
        saves: 3,
        shots: 4,
        attacking_backboard_hit_count: 2,
        double_tap_count: 1,
        goals_conceded_while_last_defender: 1,
        goal_after_kickoff,
        goal_buildup: GoalBuildupStats::default(),
    };

    let fields = stats.stat_fields();
    let shooting = find_field(&fields, "core", "shooting_percentage");

    assert_eq!(shooting.descriptor.variant, LEGACY_STAT_VARIANT);
    assert_eq!(shooting.descriptor.unit, StatUnit::Percent);
    assert_eq!(shooting.value, StatValue::Float(50.0));
    assert_eq!(
        find_field(&fields, "core", "average_goal_time_after_kickoff").value,
        StatValue::Float(10.5)
    );
    assert_eq!(
        find_field(&fields, "core", "average_goal_time_after_kickoff")
            .descriptor
            .unit,
        StatUnit::Seconds
    );
    assert_eq!(
        find_field(&fields, "core", "median_goal_time_after_kickoff").value,
        StatValue::Float(10.5)
    );
    assert_eq!(
        find_field(&fields, "core", "kickoff_goal_count").value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        find_field(&fields, "core", "attacking_backboard_hit_count").value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        find_field(&fields, "core", "double_tap_count").value,
        StatValue::Unsigned(1)
    );
}

#[test]
fn test_pressure_stats_export_includes_side_totals_and_percentages() {
    let stats = PressureStats {
        tracked_time: 10.0,
        team_zero_side_time: 4.0,
        team_one_side_time: 5.0,
        neutral_time: 1.0,
        labeled_time: LabeledFloatSums::default(),
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "pressure", "team_zero_side_time").value,
        StatValue::Float(4.0)
    );
    assert_eq!(
        find_field(&fields, "pressure", "team_one_side_time").value,
        StatValue::Float(5.0)
    );
    assert_eq!(
        find_field(&fields, "pressure", "neutral_time").value,
        StatValue::Float(1.0)
    );
    assert_eq!(
        find_field(&fields, "pressure", "team_zero_side_pct").value,
        StatValue::Float(40.0)
    );
    assert_eq!(
        find_field(&fields, "pressure", "team_one_side_pct").value,
        StatValue::Float(50.0)
    );
    assert_eq!(
        find_field(&fields, "pressure", "neutral_pct").value,
        StatValue::Float(10.0)
    );
}

#[test]
fn test_possession_stats_export_includes_neutral_fields() {
    let stats = PossessionStats {
        tracked_time: 10.0,
        team_zero_time: 4.0,
        team_one_time: 3.0,
        neutral_time: 3.0,
        labeled_time: LabeledFloatSums::default(),
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "possession", "neutral_time").value,
        StatValue::Float(3.0)
    );
    assert_eq!(
        find_field(&fields, "possession", "neutral_pct").value,
        StatValue::Float(30.0)
    );
}

#[test]
fn test_boost_stats_export_includes_respawn_and_used_fields() {
    let stats = BoostStats {
        amount_collected_big: 55.0,
        amount_collected_small: 24.0,
        amount_respawned: 68.0,
        amount_used: 91.0,
        amount_used_while_grounded: 61.0,
        amount_used_while_airborne: 30.0,
        ..BoostStats::default()
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "boost", "amount_respawned")
            .descriptor
            .unit,
        StatUnit::Boost
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_respawned").value,
        StatValue::Float(68.0)
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used").descriptor.unit,
        StatUnit::Boost
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used").value,
        StatValue::Float(91.0)
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used_while_grounded")
            .descriptor
            .unit,
        StatUnit::Boost
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used_while_grounded").value,
        StatValue::Float(61.0)
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used_while_airborne")
            .descriptor
            .unit,
        StatUnit::Boost
    );
    assert_eq!(
        find_field(&fields, "boost", "amount_used_while_airborne").value,
        StatValue::Float(30.0)
    );
}

#[test]
fn test_dodge_reset_stats_export_includes_on_ball_count() {
    let stats = DodgeResetStats {
        count: 4,
        on_ball_count: 3,
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "dodge_reset", "count").value,
        StatValue::Unsigned(4)
    );
    assert_eq!(
        find_field(&fields, "dodge_reset", "on_ball_count").value,
        StatValue::Unsigned(3)
    );
}

#[test]
fn test_touch_stats_export_includes_classification_and_ball_speed_fields() {
    let stats = TouchStats {
        touch_count: 7,
        dribble_touch_count: 2,
        control_touch_count: 1,
        medium_hit_count: 3,
        hard_hit_count: 1,
        aerial_touch_count: 2,
        high_aerial_touch_count: 1,
        last_ball_speed_change: Some(420.0),
        max_ball_speed_change: 960.0,
        cumulative_ball_speed_change: 2_100.0,
        ..TouchStats::default()
    };

    let fields = stats.stat_fields();

    assert_eq!(
        find_field(&fields, "touch", "dribble_touch_count").value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        find_field(&fields, "touch", "control_touch_count").value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        find_field(&fields, "touch", "medium_hit_count").value,
        StatValue::Unsigned(3)
    );
    assert_eq!(
        find_field(&fields, "touch", "hard_hit_count").value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        find_field(&fields, "touch", "aerial_touch_count").value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        find_field(&fields, "touch", "high_aerial_touch_count").value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        find_field(&fields, "touch", "last_ball_speed_change").value,
        StatValue::Float(420.0)
    );
    assert_eq!(
        find_field(&fields, "touch", "average_ball_speed_change")
            .descriptor
            .unit,
        StatUnit::UnrealUnitsPerSecond
    );
    assert_eq!(
        find_field(&fields, "touch", "average_ball_speed_change").value,
        StatValue::Float(300.0)
    );
    assert_eq!(
        find_field(&fields, "touch", "max_ball_speed_change").value,
        StatValue::Float(960.0)
    );
}
