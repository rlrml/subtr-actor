use super::*;
use crate::abi::{TrRotator, TrVec3};

fn scratch_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .join(format!("tem-recorder-tests-{}", std::process::id()))
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
