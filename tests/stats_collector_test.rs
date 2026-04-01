use std::path::Path;

use subtr_actor::{builtin_stats_module_names, StatsCollector};

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

#[test]
fn stats_collector_serializes_selected_modules_by_name() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let collected = StatsCollector::with_builtin_module_names(["boost", "movement"])
        .expect("builtin module selection should be valid")
        .get_stats(&replay)
        .expect("stats collection should succeed");

    let value = serde_json::to_value(&collected).expect("stats should serialize to json");
    let modules = value
        .get("modules")
        .and_then(|value| value.as_object())
        .expect("modules should serialize as an object");

    assert!(modules.contains_key("boost"));
    assert!(modules.contains_key("movement"));
    assert!(!modules.contains_key("core"));
}

#[test]
fn stats_collector_processes_all_builtin_modules() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let collected = StatsCollector::new()
        .get_stats(&replay)
        .expect("stats collection should succeed");

    let value = serde_json::to_value(&collected).expect("stats should serialize to json");
    let modules = value
        .get("modules")
        .and_then(|value| value.as_object())
        .expect("modules should serialize as an object");

    assert_eq!(modules.len(), builtin_stats_module_names().len());
    for module_name in builtin_stats_module_names() {
        assert!(
            modules.contains_key(*module_name),
            "expected serialized modules to include {module_name}"
        );
    }
}
