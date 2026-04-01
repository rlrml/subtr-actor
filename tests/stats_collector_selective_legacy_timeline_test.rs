mod common;

use subtr_actor::{StatsCollector, StatsTimelineCollector};

#[test]
fn stats_collector_selective_legacy_timeline_json_matches_old_collector() {
    let replay = common::parse_replay("assets/replays/rlcs.replay");
    let legacy = StatsCollector::with_builtin_module_names(["boost", "movement"])
        .expect("builtin module selection should be valid")
        .get_replay_stats_timeline(&replay)
        .expect("legacy compatibility timeline should build");
    let old = StatsTimelineCollector::only_modules(["boost", "movement"])
        .get_replay_data(&replay)
        .expect("old selective stats timeline should succeed");

    common::assert_replay_stats_timeline_eq(&legacy, &old);
}
