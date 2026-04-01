use super::*;

#[test]
fn movement_export_includes_labeled_tracked_time_stats() {
    let mut stats = MovementStats {
        tracked_time: 3.0,
        ..Default::default()
    };
    stats.labeled_tracked_time.add(
        [
            StatLabel::new("speed_band", "boost"),
            StatLabel::new("height_band", "low_air"),
        ],
        1.25,
    );

    let labeled_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.domain == "movement"
                && stat.descriptor.name == "tracked_time"
                && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_stats.len(), 9);
    assert_eq!(
        labeled_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("height_band", "low_air"),
                        StatLabel::new("speed_band", "boost"),
                    ]
            })
            .unwrap()
            .descriptor
            .labels,
        vec![
            StatLabel::new("height_band", "low_air"),
            StatLabel::new("speed_band", "boost"),
        ]
    );
    assert_eq!(
        labeled_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("height_band", "low_air"),
                        StatLabel::new("speed_band", "boost"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Float(1.25)
    );
    assert_eq!(
        labeled_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("height_band", "ground"),
                        StatLabel::new("speed_band", "slow"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Float(0.0)
    );
}
