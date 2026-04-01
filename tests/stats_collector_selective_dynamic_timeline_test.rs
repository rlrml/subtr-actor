mod common;

use subtr_actor::{StatsCollector, StatsTimelineCollector};

#[test]
fn stats_collector_selective_dynamic_timeline_json_matches_old_collector() {
    let replay = common::parse_replay("assets/replays/rlcs.replay");
    let dynamic = StatsCollector::with_builtin_module_names(["boost", "movement"])
        .expect("builtin module selection should be valid")
        .get_dynamic_replay_stats_timeline(&replay)
        .expect("dynamic stats timeline should build");
    let old = StatsTimelineCollector::only_modules(["boost", "movement"])
        .get_dynamic_replay_data(&replay)
        .expect("old selective dynamic stats timeline should succeed");

    common::assert_dynamic_replay_stats_timeline_eq(&dynamic, &old);
}
