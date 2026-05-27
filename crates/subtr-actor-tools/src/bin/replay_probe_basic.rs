use subtr_actor::{evaluate_replay_plausibility, StatsTimelineCollector};

use super::replay::{collect_replay_data, parse_replay};

pub(crate) fn print_metadata(path: &str) {
    let replay = parse_replay(path);
    let build_version = replay
        .properties
        .iter()
        .find(|(key, _)| key == "BuildVersion")
        .and_then(|(_, value)| value.as_string());
    let num_frames = replay
        .properties
        .iter()
        .find(|(key, _)| key == "NumFrames")
        .and_then(|(_, value)| value.as_i32());
    let match_type = replay
        .properties
        .iter()
        .find(|(key, _)| key == "MatchType")
        .and_then(|(_, value)| value.as_string());

    println!(
        "replay={path} major_version={} minor_version={} net_version={:?} build_version={:?} match_type={:?} num_frames={:?}",
        replay.major_version,
        replay.minor_version,
        replay.net_version,
        build_version,
        match_type,
        num_frames
    );
}

pub(crate) fn print_plausibility(path: &str) {
    let replay_data = collect_replay_data(path);
    let report = evaluate_replay_plausibility(&replay_data);
    println!("{report:#?}");
}

pub(crate) fn print_mechanics(path: &str) {
    let replay = parse_replay(path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .unwrap_or_else(|error| panic!("failed to collect stats timeline for {path}: {error:?}"));
    for event in &timeline.events.flick {
        println!(
            "flick {}",
            serde_json::to_string(event).expect("flick event should serialize")
        );
    }
    for event in &timeline.events.dodge_reset {
        println!(
            "dodge_reset {}",
            serde_json::to_string(event).expect("dodge-reset event should serialize")
        );
    }
    for event in &timeline.events.mechanics {
        if event.kind == "flip_reset" || event.kind == "flick" {
            println!(
                "mechanic {}",
                serde_json::to_string(event).expect("mechanic event should serialize")
            );
        }
    }
}
