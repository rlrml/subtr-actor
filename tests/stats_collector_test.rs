mod common;

use subtr_actor::{builtin_stats_module_names, StatsCollector, StatsFrameResolution};

#[test]
fn stats_collector_serializes_selected_modules_by_name() {
    let replay = common::parse_replay("assets/rlcs.replay");
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
    let replay = common::parse_replay("assets/soccar-lan.replay");
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

#[test]
fn stats_collector_captures_module_keyed_snapshot_frames() {
    let replay = common::parse_replay("assets/soccar-lan.replay");
    let snapshot = StatsCollector::with_builtin_module_names(["boost", "movement"])
        .expect("builtin module selection should be valid")
        .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 1.0 })
        .get_snapshot_data(&replay)
        .expect("snapshot collection should succeed");

    assert!(
        !snapshot.frames.is_empty(),
        "expected snapshot frames to be captured"
    );
    let final_frame = snapshot
        .frames
        .last()
        .expect("expected a final snapshot frame");
    assert!(final_frame.modules.contains_key("boost"));
    assert!(final_frame.modules.contains_key("movement"));
    assert!(!final_frame.modules.contains_key("core"));
}

#[test]
#[allow(clippy::result_large_err)]
fn stats_collector_transforms_captured_frame_modules() {
    let replay = common::parse_replay("assets/soccar-lan.replay");
    let transformed = StatsCollector::with_builtin_module_names(["boost", "movement"])
        .expect("builtin module selection should be valid")
        .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 1.0 })
        .with_module_transform(|modules| {
            let mut names = modules
                .into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<_>>();
            names.sort();
            Ok(names)
        })
        .capture_frames()
        .get_captured_data(&replay)
        .expect("transformed frame capture should succeed");

    assert!(
        !transformed.frames.is_empty(),
        "expected transformed frames to be captured"
    );
    let final_frame = transformed
        .frames
        .last()
        .expect("expected a final transformed frame");
    assert_eq!(
        final_frame.modules,
        vec!["boost".to_owned(), "movement".to_owned()]
    );
}
