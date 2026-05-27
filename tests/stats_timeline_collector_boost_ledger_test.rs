mod common;
mod stats_timeline_collector_boost_ledger;

use common::parse_replay;
use stats_timeline_collector_boost_ledger::assert_boost_ledger_reconstructs_serialized_boost_partial_sums;
use subtr_actor::*;

#[test]
fn test_boost_ledger_reconstructs_serialized_boost_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    assert!(
        !timeline.events.boost_ledger.is_empty(),
        "expected boost ledger events to be emitted"
    );
    assert!(
        !timeline.events.boost_state.is_empty(),
        "expected boost state events to be emitted"
    );
    assert_boost_ledger_reconstructs_serialized_boost_partial_sums(replay_path, &timeline);
}
