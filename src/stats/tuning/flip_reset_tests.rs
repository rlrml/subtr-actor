use super::*;
use std::path::Path;

fn sample_replay(replay_path: &str) -> FlipResetTuningReplay {
    FlipResetTuningReplay {
        replay_id: "replay-1".to_owned(),
        replay_path: replay_path.to_owned(),
        exact_dodge_refresh_count: 1,
        date: None,
        title: None,
        uploader: None,
        player_names: vec!["Player".to_owned()],
    }
}

#[test]
fn replay_path_from_manifest_resolves_relative_paths() {
    let replay = sample_replay("replays/example.replay");

    assert_eq!(
        replay.replay_path_from_manifest(Path::new("/tmp/manifest/flip-reset.json")),
        Path::new("/tmp/manifest/replays/example.replay")
    );
}

#[test]
fn replay_path_from_manifest_preserves_absolute_paths() {
    let replay = sample_replay("/tmp/replays/example.replay");

    assert_eq!(
        replay.replay_path_from_manifest(Path::new("/tmp/manifest/flip-reset.json")),
        Path::new("/tmp/replays/example.replay")
    );
}

#[test]
fn tuning_manifest_types_remain_available_from_stats_and_crate_root() {
    let replay = crate::stats::FlipResetTuningReplay {
        replay_id: "replay-1".to_owned(),
        replay_path: "replays/example.replay".to_owned(),
        exact_dodge_refresh_count: 1,
        date: None,
        title: None,
        uploader: None,
        player_names: vec!["Player".to_owned()],
    };
    let manifest = crate::FlipResetTuningManifest {
        playlist: "flip-reset".to_owned(),
        min_rank: "champion".to_owned(),
        replays: vec![replay],
    };

    assert_eq!(manifest.replays.len(), 1);
}
