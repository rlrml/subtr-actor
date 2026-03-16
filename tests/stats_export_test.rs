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
        tracked_time: 10.0,
        sum_distance_to_teammates: 2_500.0,
        sum_distance_to_ball: 20_000.0,
        sum_distance_to_ball_has_possession: 4_000.0,
        time_has_possession: 2.0,
        sum_distance_to_ball_no_possession: 15_000.0,
        time_no_possession: 5.0,
        time_most_back: 3.0,
        time_most_forward: 1.0,
        time_defensive_zone: 4.0,
        time_neutral_zone: 3.0,
        time_offensive_zone: 3.0,
        time_defensive_half: 6.0,
        time_offensive_half: 4.0,
        time_closest_to_ball: 2.0,
        time_farthest_from_ball: 4.0,
        time_even: 0.0,
        time_behind_ball: 7.0,
        time_in_front_of_ball: 3.0,
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
        find_field(&fields, "positioning", "time_defensive_zone").value,
        StatValue::Float(4.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "percent_neutral_zone").value,
        StatValue::Float(30.0)
    );
    assert_eq!(
        find_field(&fields, "positioning", "percent_most_back").value,
        StatValue::Float(30.0)
    );
}

#[test]
fn test_core_player_stats_export_uses_legacy_variant_metadata() {
    let stats = CorePlayerStats {
        score: 500,
        goals: 2,
        assists: 1,
        saves: 3,
        shots: 4,
        goals_conceded_while_last_defender: 1,
    };

    let fields = stats.stat_fields();
    let shooting = find_field(&fields, "core", "shooting_percentage");

    assert_eq!(shooting.descriptor.variant, LEGACY_STAT_VARIANT);
    assert_eq!(shooting.descriptor.unit, StatUnit::Percent);
    assert_eq!(shooting.value, StatValue::Float(50.0));
}

#[test]
fn test_boost_stats_export_includes_respawn_and_used_fields() {
    let stats = BoostStats {
        amount_collected_big: 55.0,
        amount_collected_small: 24.0,
        amount_respawned: 68.0,
        amount_used: 91.0,
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
}
