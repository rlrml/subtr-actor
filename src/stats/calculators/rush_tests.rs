use super::*;

fn rush_event(is_team_0: bool, attackers: usize, defenders: usize) -> RushEvent {
    RushEvent {
        start_time: 1.0,
        start_frame: 10,
        end_time: 2.0,
        end_frame: 20,
        is_team_0,
        attackers,
        defenders,
    }
}

#[test]
fn rush_stats_records_events_as_labeled_counts() {
    let mut stats = RushStats::default();

    stats.record(&rush_event(true, 2, 1));
    stats.record(&rush_event(true, 2, 1));
    stats.record(&rush_event(false, 3, 2));

    assert_eq!(
        stats.rush_count_with_labels(&[StatLabel::new("team", "team_zero")]),
        2
    );
    assert_eq!(
        stats.rush_count_with_labels(&[
            StatLabel::new("team", "team_zero"),
            StatLabel::new("attackers", "2"),
            StatLabel::new("defenders", "1"),
        ]),
        2
    );
    assert_eq!(
        stats.rush_count_with_labels(&[
            StatLabel::new("team", "team_one"),
            StatLabel::new("attackers", "3"),
            StatLabel::new("defenders", "2"),
        ]),
        1
    );
}

#[test]
fn rush_stats_keeps_legacy_fields_synced_from_labeled_counts() {
    let mut stats = RushStats::default();

    stats.record(&rush_event(true, 2, 1));
    stats.record(&rush_event(false, 3, 3));

    assert_eq!(stats.team_zero_count, 1);
    assert_eq!(stats.team_zero_two_v_one_count, 1);
    assert_eq!(stats.team_zero_two_v_two_count, 0);
    assert_eq!(stats.team_one_count, 1);
    assert_eq!(stats.team_one_three_v_three_count, 1);
    assert_eq!(
        stats.for_team(false),
        RushTeamStats {
            count: 1,
            three_v_three_count: 1,
            ..Default::default()
        }
    );
}
