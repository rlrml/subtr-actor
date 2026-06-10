use super::*;

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

#[test]
fn touch_export_includes_role_and_play_depth_labeled_counts() {
    let mut stats = TouchStats::default();
    stats
        .touch_counts_by_role
        .increment([RoleState::SecondMan.as_label()]);
    stats
        .touch_counts_by_play_depth
        .increment([PlayDepthState::BehindPlay.as_label()]);

    let role_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| stat.descriptor.name == "role_touch_count")
        .collect();
    assert_eq!(role_stats.len(), 5);
    assert_eq!(
        role_stats
            .iter()
            .find(|stat| stat.descriptor.labels == vec![StatLabel::new("role", "second_man")])
            .unwrap()
            .value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        role_stats
            .iter()
            .find(|stat| stat.descriptor.labels == vec![StatLabel::new("role", "first_man")])
            .unwrap()
            .value,
        StatValue::Unsigned(0)
    );

    let depth_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| stat.descriptor.name == "play_depth_touch_count")
        .collect();
    assert_eq!(depth_stats.len(), 4);
    assert_eq!(
        depth_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels == vec![StatLabel::new("play_depth", "behind_play")]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(1)
    );
    assert_eq!(
        depth_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels == vec![StatLabel::new("play_depth", "ahead_of_play")]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(0)
    );
}
