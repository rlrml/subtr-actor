use serde_json::{Value, json};

use super::{Archetype, BallSpawn, CarSpawn, PlayerCarSpawn};
use crate::container::TrainingFile;
use crate::pack::{Round, TrainingPack};

// Synthetic corpus modeled on the three shapes observed in real packs.
const BALL: &str = r#"{"ObjectArchetype":"Archetypes.Ball.Ball_GameEditor","StartLocationX":12.3400,"StartLocationY":4300.2100,"StartLocationZ":650.0000,"VelocityStartRotationP":-391,"VelocityStartRotationY":16241,"VelocityStartRotationR":0,"VelocityStartSpeed":2100.5000}"#;
const CAR_SPAWN: &str = r#"{"ObjectArchetype":"Archetypes.GameEditor.DynamicSpawnPointMesh","LocationX":0.0000,"LocationY":0.0000,"LocationZ":30.0000,"RotationP":0,"RotationY":16384,"RotationR":0,"VelocityStartSpeed":0.0000}"#;
const CAR_SPAWN_NO_SPEED: &str = r#"{"ObjectArchetype":"Archetypes.GameEditor.DynamicSpawnPointMesh","LocationX":139.6400,"LocationY":1767.9099,"LocationZ":17.0100,"RotationP":-100,"RotationY":21707,"RotationR":1}"#;
const PLAYER_CAR: &str = r#"{"IsPC":true,"LocationX":-500.2500,"LocationY":-800.7500,"LocationZ":17.0100,"RotationP":-837,"RotationY":3634,"RotationR":0}"#;
// Psyonix-made packs write zeros as bare ints.
const PLAYER_CAR_INT_ZEROS: &str = r#"{"IsPC":true,"LocationX":0,"LocationY":0,"LocationZ":0,"RotationP":0,"RotationY":0,"RotationR":0}"#;

/// Numeric-equality JSON comparison: `0` == `0.0000`.
fn semantically_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, value)| {
                    b.get(key)
                        .is_some_and(|other| semantically_equal(value, other))
                })
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(value, other)| semantically_equal(value, other))
        }
        _ => a == b,
    }
}

fn assert_semantic_roundtrip(raw: &str) {
    let regenerated = Archetype::parse(raw).to_archetype_string();
    let original: Value = serde_json::from_str(raw).unwrap();
    let reparsed: Value = serde_json::from_str(&regenerated).unwrap();
    assert!(
        semantically_equal(&original, &reparsed),
        "semantic mismatch:\n  original:    {raw}\n  regenerated: {regenerated}"
    );
}

#[test]
fn parses_the_ball_shape() {
    let Archetype::Ball(ball) = Archetype::parse(BALL) else {
        panic!("expected a ball");
    };
    assert_eq!(ball.start_location_x, 12.34);
    assert_eq!(ball.start_location_y, 4300.21);
    assert_eq!(ball.start_location_z, 650.0);
    assert_eq!(ball.velocity_start_rotation_p, -391);
    assert_eq!(ball.velocity_start_rotation_y, 16241);
    assert_eq!(ball.velocity_start_rotation_r, 0);
    assert_eq!(ball.velocity_start_speed, 2100.5);
    assert!(ball.extras.is_empty());
}

#[test]
fn parses_car_spawns_with_and_without_velocity_start_speed() {
    let Archetype::CarSpawnPoint(with_speed) = Archetype::parse(CAR_SPAWN) else {
        panic!("expected a car spawn point");
    };
    assert_eq!(with_speed.velocity_start_speed, Some(0.0));
    assert_eq!(with_speed.rotation_y, 16384);

    let Archetype::CarSpawnPoint(without_speed) = Archetype::parse(CAR_SPAWN_NO_SPEED) else {
        panic!("expected a car spawn point");
    };
    assert_eq!(without_speed.velocity_start_speed, None);
    assert_eq!(without_speed.location_x, 139.64);
    assert_eq!(without_speed.rotation_r, 1);
}

#[test]
fn parses_player_cars_including_bare_int_zeros() {
    let Archetype::PlayerCar(player_car) = Archetype::parse(PLAYER_CAR) else {
        panic!("expected a player car");
    };
    assert!(player_car.is_pc);
    assert_eq!(player_car.location_x, Some(-500.25));
    assert_eq!(player_car.rotation_y, Some(3634));

    let Archetype::PlayerCar(zeros) = Archetype::parse(PLAYER_CAR_INT_ZEROS) else {
        panic!("expected a player car");
    };
    assert_eq!(zeros.location_x, Some(0.0));
    assert_eq!(zeros.rotation_p, Some(0));
}

#[test]
fn parses_a_bare_is_pc_entry_with_no_transform() {
    // Real game output (and the committed synthetic fixture) includes
    // player car entries with no transform keys at all.
    let raw = r#"{"IsPC":true}"#;
    let Archetype::PlayerCar(player_car) = Archetype::parse(raw) else {
        panic!("expected a player car");
    };
    assert!(player_car.is_pc);
    assert_eq!(player_car.location_x, None);
    assert_eq!(player_car.rotation_r, None);
    assert!(player_car.extras.is_empty());
    // Absent keys stay absent on regeneration.
    let regenerated = Archetype::PlayerCar(player_car).to_archetype_string();
    assert_eq!(regenerated, raw);
}

#[test]
fn unrecognized_strings_fall_back_to_unknown_verbatim() {
    for raw in [
        "not json at all",
        "[1,2,3]",
        r#"{"ObjectArchetype":"Archetypes.Future.NewThing","LocationX":1.0000}"#,
        r#"{"SomethingElse":1}"#,
        // A recognized ObjectArchetype with a missing required key.
        r#"{"ObjectArchetype":"Archetypes.Ball.Ball_GameEditor","StartLocationX":1.0000}"#,
        // A mistyped key (present but not a number).
        r#"{"IsPC":true,"LocationX":"oops","LocationY":0,"LocationZ":0,"RotationP":0,"RotationY":0,"RotationR":0}"#,
    ] {
        let parsed = Archetype::parse(raw);
        assert_eq!(
            parsed,
            Archetype::Unknown {
                raw: raw.to_string()
            },
            "expected Unknown for {raw}"
        );
        assert_eq!(parsed.to_archetype_string(), raw);
    }
}

#[test]
fn unknown_keys_survive_parse_then_serialize() {
    let raw = r#"{"ObjectArchetype":"Archetypes.Ball.Ball_GameEditor","StartLocationX":1.0000,"StartLocationY":2.0000,"StartLocationZ":3.0000,"VelocityStartRotationP":4,"VelocityStartRotationY":5,"VelocityStartRotationR":6,"VelocityStartSpeed":7.0000,"ZFuture":true,"AFuture":{"nested":1}}"#;
    let Archetype::Ball(ball) = Archetype::parse(raw) else {
        panic!("expected a ball");
    };
    assert_eq!(ball.extras.len(), 2);
    assert_eq!(ball.extras.get("ZFuture"), Some(&json!(true)));
    assert_eq!(ball.extras.get("AFuture"), Some(&json!({"nested": 1})));

    let regenerated = Archetype::Ball(ball).to_archetype_string();
    // Extras come after the known keys, in alphabetical order (serde_json's
    // sorted Map).
    assert_eq!(
        regenerated,
        r#"{"ObjectArchetype":"Archetypes.Ball.Ball_GameEditor","StartLocationX":1.0000,"StartLocationY":2.0000,"StartLocationZ":3.0000,"VelocityStartRotationP":4,"VelocityStartRotationY":5,"VelocityStartRotationR":6,"VelocityStartSpeed":7.0000,"AFuture":{"nested":1},"ZFuture":true}"#
    );
    assert_semantic_roundtrip(raw);
}

#[test]
fn regenerated_strings_use_game_key_order_and_formatting() {
    assert_eq!(Archetype::parse(BALL).to_archetype_string(), BALL);
    assert_eq!(Archetype::parse(CAR_SPAWN).to_archetype_string(), CAR_SPAWN);
    assert_eq!(
        Archetype::parse(CAR_SPAWN_NO_SPEED).to_archetype_string(),
        CAR_SPAWN_NO_SPEED
    );
    assert_eq!(
        Archetype::parse(PLAYER_CAR).to_archetype_string(),
        PLAYER_CAR
    );
    // Bare-int zeros regenerate as 4-decimal floats: semantically equal but
    // not byte-equal.
    assert_eq!(
        Archetype::parse(PLAYER_CAR_INT_ZEROS).to_archetype_string(),
        r#"{"IsPC":true,"LocationX":0.0000,"LocationY":0.0000,"LocationZ":0.0000,"RotationP":0,"RotationY":0,"RotationR":0}"#
    );
    for raw in [
        BALL,
        CAR_SPAWN,
        CAR_SPAWN_NO_SPEED,
        PLAYER_CAR,
        PLAYER_CAR_INT_ZEROS,
    ] {
        assert_semantic_roundtrip(raw);
    }
}

#[test]
fn default_constructors_match_the_fresh_editor_values() {
    let ball = BallSpawn::default();
    assert_eq!(
        (
            ball.start_location_x,
            ball.start_location_y,
            ball.start_location_z
        ),
        (0.0, 4120.0, 100.4872)
    );
    assert_eq!(
        (
            ball.velocity_start_rotation_p,
            ball.velocity_start_rotation_y,
            ball.velocity_start_rotation_r
        ),
        (8191, -16384, 0)
    );
    assert_eq!(ball.velocity_start_speed, 1500.0);

    let car = CarSpawn::default();
    assert_eq!(
        (car.location_x, car.location_y, car.location_z),
        (0.0, 0.0, 30.0)
    );
    assert_eq!(
        (car.rotation_p, car.rotation_y, car.rotation_r),
        (0, 16384, 0)
    );
    assert_eq!(car.velocity_start_speed, Some(0.0));

    let player_car = PlayerCarSpawn::default();
    assert!(player_car.is_pc);
    assert_eq!(player_car.location_x, Some(0.0));
    assert_eq!(player_car.rotation_y, Some(0));
}

#[test]
fn archetype_serde_json_roundtrip_is_a_tagged_union() {
    let archetypes = vec![
        Archetype::Ball(BallSpawn::default()),
        Archetype::CarSpawnPoint(CarSpawn::default()),
        Archetype::PlayerCar(PlayerCarSpawn::default()),
        Archetype::Unknown {
            raw: "garbage".to_string(),
        },
    ];
    let json = serde_json::to_string(&archetypes).unwrap();
    assert!(json.contains(r#""kind":"Ball""#));
    assert!(json.contains(r#""kind":"CarSpawnPoint""#));
    assert!(json.contains(r#""kind":"PlayerCar""#));
    assert!(json.contains(r#""kind":"Unknown""#));
    let back: Vec<Archetype> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, archetypes);
}

// --- TrainingFile editing ---

fn sample_file() -> TrainingFile {
    TrainingFile::from_pack(&TrainingPack {
        rounds: vec![
            Round {
                time_limit: 8.0,
                serialized_archetypes: vec![
                    BALL.to_string(),
                    CAR_SPAWN.to_string(),
                    PLAYER_CAR.to_string(),
                ],
            },
            Round {
                time_limit: 0.0,
                serialized_archetypes: vec![CAR_SPAWN_NO_SPEED.to_string()],
            },
        ],
        ..TrainingPack::default()
    })
    .unwrap()
}

#[test]
fn round_archetypes_parses_each_entry() {
    let file = sample_file();
    let archetypes = file.round_archetypes(0).unwrap();
    assert_eq!(archetypes.len(), 3);
    assert!(matches!(archetypes[0], Archetype::Ball(_)));
    assert!(matches!(archetypes[1], Archetype::CarSpawnPoint(_)));
    assert!(matches!(archetypes[2], Archetype::PlayerCar(_)));
    assert!(file.round_archetypes(2).is_err());
}

#[test]
fn set_add_and_remove_round_archetypes() {
    let mut file = sample_file();

    let car = CarSpawn {
        location_x: 512.5,
        ..CarSpawn::default()
    };
    file.set_round_archetype(0, 1, &Archetype::CarSpawnPoint(car.clone()))
        .unwrap();
    let archetypes = file.round_archetypes(0).unwrap();
    assert_eq!(archetypes[1], Archetype::CarSpawnPoint(car));
    // The neighbors were not rewritten.
    let round = file.rounds().unwrap().remove(0);
    assert_eq!(round.serialized_archetypes[0], BALL);
    assert_eq!(round.serialized_archetypes[2], PLAYER_CAR);

    file.add_round_archetype(0, &Archetype::PlayerCar(PlayerCarSpawn::default()))
        .unwrap();
    assert_eq!(file.round_archetypes(0).unwrap().len(), 4);

    let removed = file.remove_round_archetype(0, 3).unwrap();
    assert_eq!(removed, Archetype::PlayerCar(PlayerCarSpawn::default()));
    assert_eq!(file.round_archetypes(0).unwrap().len(), 3);

    // Out-of-range indices error.
    assert!(file.set_round_archetype(0, 9, &removed).is_err());
    assert!(file.remove_round_archetype(0, 9).is_err());
    assert!(file.add_round_archetype(9, &removed).is_err());
}

#[test]
fn removing_the_last_archetype_drops_the_property() {
    let mut file = sample_file();
    file.remove_round_archetype(1, 0).unwrap();
    assert!(file.round_archetypes(1).unwrap().is_empty());
    // Round 1 had no TimeLimit either, so it serializes as an empty round;
    // the file must still encode and re-parse.
    let bytes = file.to_bytes().unwrap();
    let reparsed = TrainingFile::from_bytes(&bytes).unwrap();
    assert!(
        reparsed.pack().unwrap().rounds[1]
            .serialized_archetypes
            .is_empty()
    );
}

#[test]
fn set_round_ball_replaces_the_first_ball_or_inserts_at_front() {
    let mut file = sample_file();

    // Replace path: round 0 already has a ball at index 0.
    let ball = BallSpawn {
        start_location_z: 901.25,
        ..BallSpawn::default()
    };
    file.set_round_ball(0, &ball).unwrap();
    let archetypes = file.round_archetypes(0).unwrap();
    assert_eq!(archetypes.len(), 3);
    assert_eq!(archetypes[0], Archetype::Ball(ball.clone()));

    // Insert path: round 1 has no ball; it goes to position 0.
    file.set_round_ball(1, &ball).unwrap();
    let archetypes = file.round_archetypes(1).unwrap();
    assert_eq!(archetypes.len(), 2);
    assert_eq!(archetypes[0], Archetype::Ball(ball));
    assert!(matches!(archetypes[1], Archetype::CarSpawnPoint(_)));

    assert!(file.set_round_ball(9, &BallSpawn::default()).is_err());
}

#[test]
fn set_round_time_limit_edits_in_place_and_zero_removes() {
    let mut file = sample_file();

    file.set_round_time_limit(0, 30.0).unwrap();
    assert_eq!(file.rounds().unwrap()[0].time_limit, 30.0);

    // Round 1 has no TimeLimit; setting one must place it before
    // SerializedArchetypes (the game's order) and survive a byte round trip.
    file.set_round_time_limit(1, 5.5).unwrap();
    let bytes = file.to_bytes().unwrap();
    let reparsed = TrainingFile::from_bytes(&bytes).unwrap();
    assert_eq!(reparsed.pack().unwrap().rounds[1].time_limit, 5.5);

    // Zero removes the property (the game's omit-default convention).
    file.set_round_time_limit(0, 0.0).unwrap();
    assert_eq!(file.rounds().unwrap()[0].time_limit, 0.0);

    assert!(file.set_round_time_limit(9, 1.0).is_err());
}

#[test]
fn edited_archetypes_roundtrip_semantically_through_tem_bytes() {
    let mut file = sample_file();
    let ball = BallSpawn {
        start_location_x: -62.16,
        velocity_start_speed: 2734.3478,
        ..BallSpawn::default()
    };
    file.set_round_ball(0, &ball).unwrap();

    let bytes = file.to_bytes().unwrap();
    let reparsed = TrainingFile::from_bytes(&bytes).unwrap();
    let edited = &reparsed.pack().unwrap().rounds[0].serialized_archetypes[0];
    let expected = Archetype::Ball(ball).to_archetype_string();
    let edited_json: Value = serde_json::from_str(edited).unwrap();
    let expected_json: Value = serde_json::from_str(&expected).unwrap();
    assert!(semantically_equal(&edited_json, &expected_json));
}

const SYNTHETIC: &[u8] = include_bytes!("../assets/synthetic-pack.tem");

#[test]
fn untouched_rounds_stay_byte_identical_after_editing_another_round() {
    let original = TrainingFile::from_bytes(SYNTHETIC).unwrap();
    let original_rounds = original.pack().unwrap().rounds;

    let mut edited = TrainingFile::from_bytes(SYNTHETIC).unwrap();
    edited.set_round_ball(1, &BallSpawn::default()).unwrap();
    edited.set_round_time_limit(1, 42.0).unwrap();

    let reparsed = TrainingFile::from_bytes(&edited.to_bytes().unwrap()).unwrap();
    let reparsed_rounds = reparsed.pack().unwrap().rounds;
    assert_eq!(reparsed_rounds.len(), original_rounds.len());
    for index in [0, 2] {
        // Byte-identical strings (and time limits) for the rounds that were
        // not edited — including the original float formatting.
        assert_eq!(
            reparsed_rounds[index].serialized_archetypes,
            original_rounds[index].serialized_archetypes,
            "round {index} was rewritten"
        );
        assert_eq!(
            reparsed_rounds[index].time_limit,
            original_rounds[index].time_limit
        );
    }
    // Within the edited round, the archetype that was not modified is also
    // byte-identical.
    assert_eq!(
        reparsed_rounds[1].serialized_archetypes.last(),
        original_rounds[1].serialized_archetypes.first(),
    );
}
