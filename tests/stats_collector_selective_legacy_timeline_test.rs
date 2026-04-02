mod common;

use subtr_actor::{StatsCollector, StatsTimelineCollector};

#[test]
fn stats_collector_full_typed_timeline_matches_stats_timeline_collector() {
    let replay = common::parse_replay("assets/replays/rlcs.replay");
    let legacy = StatsCollector::new()
        .get_replay_stats_timeline(&replay)
        .expect("stats collector typed timeline should build");
    let old = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("stats timeline collector should build");

    common::assert_replay_stats_timeline_eq(&legacy, &old);
}
