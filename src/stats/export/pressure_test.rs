use super::*;

#[test]
fn pressure_export_includes_labeled_time_stats() {
    let mut stats = PressureStats {
        tracked_time: 4.0,
        ..Default::default()
    };
    stats
        .labeled_time
        .add([StatLabel::new("field_half", "team_zero_side")], 1.5);

    let labeled_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.domain == "pressure"
                && stat.descriptor.name == "time"
                && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_stats.len(), 1);
    assert_eq!(
        labeled_stats[0].descriptor.labels,
        vec![StatLabel::new("field_half", "team_zero_side")]
    );
    assert_eq!(labeled_stats[0].value, StatValue::Float(1.5));
}
