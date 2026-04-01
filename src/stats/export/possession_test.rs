use super::*;

#[test]
fn possession_export_includes_labeled_time_stats() {
    let mut stats = PossessionStats {
        tracked_time: 5.0,
        ..Default::default()
    };
    stats
        .labeled_time
        .add([StatLabel::new("possession_state", "team_zero")], 2.5);

    let labeled_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.domain == "possession"
                && stat.descriptor.name == "time"
                && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_stats.len(), 1);
    assert_eq!(
        labeled_stats[0].descriptor.labels,
        vec![StatLabel::new("possession_state", "team_zero")]
    );
    assert_eq!(labeled_stats[0].value, StatValue::Float(2.5));
}
