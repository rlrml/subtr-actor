use super::*;
use crate::abi::{TrRotator, TrVec3};

fn scratch_path(name: &str) -> std::path::PathBuf {
    // Each scratch file gets its own subdirectory so a test's cleanup
    // (`remove_dir_all` on the parent) cannot delete another test's files
    // when the suite runs in parallel. `name` is unique per test.
    std::env::temp_dir()
        .join(format!("replay-to-training-tests-{}", std::process::id()))
        .join(name.trim_end_matches(".Tem"))
        .join(name)
}

fn sample_ball() -> TrBallState {
    TrBallState {
        location: TrVec3 {
            x: 24.5,
            y: 4269.25,
            z: 224.4375,
        },
        linear_velocity: TrVec3 {
            x: 500.0,
            y: 0.0,
            z: 250.0,
        },
        angular_velocity: TrVec3::default(),
    }
}

fn sample_car() -> TrCarState {
    TrCarState {
        location: TrVec3 {
            x: -600.0,
            y: -700.0,
            z: 17.0,
        },
        rotation: TrRotator {
            pitch: -837,
            yaw: 3634,
            roll: 0,
        },
        boost_amount: 0.33,
        is_primary: 1,
        team: 0,
        ..TrCarState::default()
    }
}

#[test]
fn fresh_pack_has_defaults_and_unique_guid() {
    let first = RecorderPack::new();
    let second = RecorderPack::new();
    let pack = first.pack().unwrap();
    assert_eq!(pack.name.as_deref(), Some(DEFAULT_PACK_NAME));
    assert_eq!(pack.map_name.as_deref(), Some(DEFAULT_MAP_NAME));
    assert!(pack.created_at > 0);
    assert_eq!(first.shot_count(), 0);
    assert_ne!(first.guid_hex(), second.guid_hex());
    assert_eq!(first.guid_hex().len(), 32);
}

#[test]
fn add_and_remove_shots_updates_rounds() {
    let mut recorder = RecorderPack::new();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 10.0)
        .unwrap();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 6.0)
        .unwrap();
    assert_eq!(recorder.shot_count(), 2);

    let rounds = recorder.pack().unwrap().rounds;
    assert_eq!(rounds[0].time_limit, 10.0);
    assert_eq!(rounds[0].serialized_archetypes.len(), 3);
    assert!(rounds[0].serialized_archetypes[0].contains("Ball_GameEditor"));

    recorder.remove_shot(0).unwrap();
    assert_eq!(recorder.shot_count(), 1);
    assert_eq!(recorder.pack().unwrap().rounds[0].time_limit, 6.0);

    assert!(recorder.remove_shot(5).is_err());
}

#[test]
fn shot_summary_reports_ball_location_and_car_count() {
    let mut recorder = RecorderPack::new();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 10.0)
        .unwrap();
    let summary = recorder.shot_summary(0).unwrap();
    // `{:.0}` uses round-half-to-even, so 24.5 renders as 24.
    assert_eq!(summary, "ball (24, 4269, 224), 1 car, 10s");
    assert!(recorder.shot_summary(1).is_none());
}

#[test]
fn save_and_reopen_round_trips_shots_and_metadata() {
    let mut recorder = RecorderPack::new();
    recorder.set_name(Some("Round Trip Pack")).unwrap();
    recorder.set_creator_name(Some("subtr-actor")).unwrap();
    recorder
        .set_difficulty(&subtr_actor_training::Difficulty::Hard)
        .unwrap();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 8.0)
        .unwrap();
    let original_rounds = recorder.pack().unwrap().rounds;

    let path = scratch_path("round-trip.Tem");
    recorder.save(&path).unwrap();

    let mut reopened = RecorderPack::open(&path).unwrap();
    let pack = reopened.pack().unwrap();
    assert_eq!(pack.name.as_deref(), Some("Round Trip Pack"));
    assert_eq!(pack.creator_name.as_deref(), Some("subtr-actor"));
    assert_eq!(pack.difficulty, subtr_actor_training::Difficulty::Hard);
    assert_eq!(pack.rounds, original_rounds);

    // Appending to a reopened pack keeps the existing rounds.
    reopened
        .add_shot(&sample_ball(), &[sample_car()], 4.0)
        .unwrap();
    assert_eq!(reopened.shot_count(), 2);

    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// The core non-destructive guarantee: seed memory from an existing target,
/// capture more shots, save back to the target — the original rounds survive
/// unchanged and the new one is appended (count grows by exactly the number
/// captured).
#[test]
fn target_save_preserves_originals_and_appends() {
    // Author a target with two original shots and save it to disk.
    let path = scratch_path("target-append.Tem");
    {
        let mut author = RecorderPack::new();
        author.set_name(Some("Target Pack")).unwrap();
        author
            .add_shot(&sample_ball(), &[sample_car()], 10.0)
            .unwrap();
        author
            .add_shot(&sample_ball(), &[sample_car()], 6.0)
            .unwrap();
        author.save(&path).unwrap();
    }
    let original = RecorderPack::open(&path).unwrap().pack().unwrap();
    assert_eq!(original.rounds.len(), 2);

    // Set the target = open it into memory, then capture a third shot.
    let mut recorder = RecorderPack::open(&path).unwrap();
    assert_eq!(recorder.loaded_from(), Some(path.as_path()));
    assert_eq!(recorder.shot_count(), 2);
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 4.0)
        .unwrap();
    assert_eq!(recorder.shot_count(), 3);

    // Saving back to the same target is an Appended (non-destructive) write.
    assert_eq!(
        recorder.save_to_target(&path).unwrap(),
        TargetSaveOutcome::Appended
    );

    // Reopen the target from disk: the two originals are byte-identical and
    // the new shot is present — 3 total, no double-counting.
    let reopened = RecorderPack::open(&path).unwrap().pack().unwrap();
    assert_eq!(reopened.rounds.len(), 3);
    assert_eq!(reopened.rounds[0], original.rounds[0]);
    assert_eq!(reopened.rounds[1], original.rounds[1]);
    assert_eq!(reopened.rounds[2].time_limit, 4.0);
    assert_eq!(reopened.guid, original.guid);

    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// Saving to a brand-new target path reports `Created` and writes the file.
#[test]
fn target_save_to_new_path_creates_file() {
    let path = scratch_path("target-create.Tem");
    let mut recorder = RecorderPack::new();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 8.0)
        .unwrap();
    assert_eq!(
        recorder.classify_target_save(&path).unwrap(),
        TargetSaveOutcome::Created
    );
    assert_eq!(
        recorder.save_to_target(&path).unwrap(),
        TargetSaveOutcome::Created
    );
    // After the first save the path is remembered, so a re-save is Appended.
    assert_eq!(
        recorder.save_to_target(&path).unwrap(),
        TargetSaveOutcome::Appended
    );
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// The clobber guard: a fresh (un-loaded) pack refuses to overwrite a target
/// path that already holds a different pack, and writes nothing.
#[test]
fn target_save_refuses_to_clobber_a_different_pack() {
    let path = scratch_path("target-foreign.Tem");
    // A foreign pack already lives at the target path.
    let foreign = {
        let mut other = RecorderPack::new();
        other.set_name(Some("Foreign Pack")).unwrap();
        other
            .add_shot(&sample_ball(), &[sample_car()], 9.0)
            .unwrap();
        other.save(&path).unwrap();
        other.pack().unwrap()
    };

    // A fresh in-memory pack (different GUID, not loaded from the path) must
    // refuse rather than overwrite.
    let mut recorder = RecorderPack::new();
    recorder
        .add_shot(&sample_ball(), &[sample_car()], 3.0)
        .unwrap();
    assert_ne!(
        recorder.guid_hex(),
        RecorderPack::open(&path).unwrap().guid_hex()
    );
    assert_eq!(
        recorder.classify_target_save(&path).unwrap(),
        TargetSaveOutcome::RefusedDifferentPack
    );
    assert_eq!(
        recorder.save_to_target(&path).unwrap(),
        TargetSaveOutcome::RefusedDifferentPack
    );

    // The foreign pack on disk is untouched.
    let after = RecorderPack::open(&path).unwrap().pack().unwrap();
    assert_eq!(after.guid, foreign.guid);
    assert_eq!(after.rounds, foreign.rounds);
    assert_eq!(after.name.as_deref(), Some("Foreign Pack"));

    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// `file_guid_hex` reads a target's GUID, distinguishes missing from
/// present, and matches the recorder's own hex formatting.
#[test]
fn file_guid_hex_reads_existing_and_reports_missing() {
    let path = scratch_path("guid-probe.Tem");
    assert_eq!(file_guid_hex(&path).unwrap(), None);
    let mut recorder = RecorderPack::new();
    recorder.save(&path).unwrap();
    assert_eq!(
        file_guid_hex(&path).unwrap().as_deref(),
        Some(recorder.guid_hex().as_str())
    );
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn open_rejects_garbage_files() {
    let path = scratch_path("garbage.Tem");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"not a tem file").unwrap();
    assert!(RecorderPack::open(&path).is_err());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn json_number_field_extracts_flat_fields() {
    let line = "{\"IsPC\":true,\"LocationX\":-599.9999,\"RotationY\":3634}";
    assert_eq!(json_number_field(line, "LocationX"), Some(-599.9999));
    assert_eq!(json_number_field(line, "RotationY"), Some(3634.0));
    assert_eq!(json_number_field(line, "Missing"), None);
}
