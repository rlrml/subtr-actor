use super::*;
use crate::{StatLabel, StatValue, LABELED_STAT_VARIANT};

#[test]
fn touch_export_includes_labeled_touch_count_stats() {
    let mut stats = TouchStats {
        touch_count: 2,
        ..Default::default()
    };
    stats.labeled_touch_counts.increment([
        StatLabel::new("kind", "hard_hit"),
        StatLabel::new("height_band", "high_air"),
        StatLabel::new("surface", "air"),
        StatLabel::new("dodge_state", "dodge"),
    ]);
    stats.labeled_touch_counts.increment([
        StatLabel::new("kind", "hard_hit"),
        StatLabel::new("height_band", "high_air"),
        StatLabel::new("surface", "air"),
        StatLabel::new("dodge_state", "dodge"),
    ]);

    let labeled_touch_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.name == "touch_count" && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_touch_stats.len(), 54);
    assert_eq!(
        labeled_touch_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("dodge_state", "dodge"),
                        StatLabel::new("height_band", "high_air"),
                        StatLabel::new("kind", "hard_hit"),
                        StatLabel::new("surface", "air"),
                    ]
            })
            .unwrap()
            .descriptor
            .labels,
        vec![
            StatLabel::new("dodge_state", "dodge"),
            StatLabel::new("height_band", "high_air"),
            StatLabel::new("kind", "hard_hit"),
            StatLabel::new("surface", "air"),
        ]
    );
    assert_eq!(
        labeled_touch_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("dodge_state", "dodge"),
                        StatLabel::new("height_band", "high_air"),
                        StatLabel::new("kind", "hard_hit"),
                        StatLabel::new("surface", "air"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        labeled_touch_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("dodge_state", "no_dodge"),
                        StatLabel::new("height_band", "ground"),
                        StatLabel::new("kind", "control"),
                        StatLabel::new("surface", "ground"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(0)
    );
}
