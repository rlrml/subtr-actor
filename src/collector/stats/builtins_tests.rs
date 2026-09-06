use super::*;

#[test]
fn core_snapshot_preserves_caught_ahead_goal_count() {
    let stats = CorePlayerStats {
        scoring_context: PlayerScoringContextStats {
            caught_ahead_of_play_on_conceded_goals: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let snapshot = serde_json::to_value(CorePlayerStatsSnapshot::from(&stats))
        .expect("snapshot should serialize");
    assert_eq!(snapshot["caught_ahead_of_play_on_conceded_goals"], 2);
    let restored: CorePlayerStats =
        serde_json::from_value(snapshot).expect("snapshot should preserve core stats");
    assert_eq!(restored, stats);
}
