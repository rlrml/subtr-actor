mod common;

use subtr_actor::{StatsCollector, StatsTimelineCollector};

#[test]
fn stats_collector_legacy_timeline_json_matches_old_collector() {
    let replay = common::parse_replay("assets/replays/rlcs.replay");
    let legacy = StatsCollector::new()
        .get_replay_stats_timeline(&replay)
        .expect("legacy compatibility timeline should build");
    let old = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("old stats timeline should succeed");

    common::assert_replay_stats_timeline_eq(&legacy, &old);
}
