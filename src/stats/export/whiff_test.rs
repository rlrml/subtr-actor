use super::*;

#[test]
fn whiff_export_includes_labeled_whiff_count_stats() {
    let mut stats = WhiffStats::default();
    stats.labeled_whiff_counts.increment([
        StatLabel::new("vertical_state", "aerial"),
        StatLabel::new("dodge_state", "dodge"),
    ]);
    stats.labeled_whiff_counts.increment([
        StatLabel::new("vertical_state", "aerial"),
        StatLabel::new("dodge_state", "dodge"),
    ]);

    let labeled_whiff_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.domain == "whiff"
                && stat.descriptor.name == "whiff_count"
                && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_whiff_stats.len(), 4);
    assert_eq!(
        labeled_whiff_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("dodge_state", "dodge"),
                        StatLabel::new("vertical_state", "aerial"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        labeled_whiff_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("dodge_state", "no_dodge"),
                        StatLabel::new("vertical_state", "grounded"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(0)
    );
}
