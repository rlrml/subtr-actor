use super::*;
use crate::{StatLabel, StatValue, LABELED_STAT_VARIANT};

#[test]
fn rush_export_includes_labeled_rush_count_stats() {
    let mut stats = RushStats::default();
    stats.labeled_rush_counts.increment([
        StatLabel::new("team", "team_zero"),
        StatLabel::new("attackers", "2"),
        StatLabel::new("defenders", "1"),
    ]);
    stats.labeled_rush_counts.increment([
        StatLabel::new("team", "team_zero"),
        StatLabel::new("attackers", "2"),
        StatLabel::new("defenders", "1"),
    ]);

    let labeled_rush_stats: Vec<_> = stats
        .stat_fields()
        .into_iter()
        .filter(|stat| {
            stat.descriptor.domain == "rush"
                && stat.descriptor.name == "rush_count"
                && stat.descriptor.variant == LABELED_STAT_VARIANT
        })
        .collect();

    assert_eq!(labeled_rush_stats.len(), 12);
    assert_eq!(
        labeled_rush_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("attackers", "2"),
                        StatLabel::new("defenders", "1"),
                        StatLabel::new("team", "team_zero"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(2)
    );
    assert_eq!(
        labeled_rush_stats
            .iter()
            .find(|stat| {
                stat.descriptor.labels
                    == vec![
                        StatLabel::new("attackers", "3"),
                        StatLabel::new("defenders", "3"),
                        StatLabel::new("team", "team_one"),
                    ]
            })
            .unwrap()
            .value,
        StatValue::Unsigned(0)
    );
}
