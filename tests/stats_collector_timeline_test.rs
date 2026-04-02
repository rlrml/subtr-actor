mod common;

use subtr_actor::{StatsCollector, StatsTimelineCollector};

#[test]
fn stats_collector_timeline_matches_stats_timeline_collector() {
    let replay = common::parse_replay("assets/replays/rlcs.replay");
    let timeline = StatsCollector::new()
        .get_replay_stats_timeline(&replay)
        .expect("stats collector timeline should build");
    let stats_timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("stats timeline collector should succeed");

    common::assert_replay_stats_timeline_eq(&timeline, &stats_timeline);
}
