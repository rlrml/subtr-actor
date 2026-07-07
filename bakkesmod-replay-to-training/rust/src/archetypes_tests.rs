use super::*;
use crate::abi::{TrBallState, TrCarState, TrRotator};

fn vec3(x: f32, y: f32, z: f32) -> TrVec3 {
    TrVec3 { x, y, z }
}

#[test]
fn ball_archetype_with_zero_velocity_matches_corpus_shape() {
    let ball = TrBallState {
        location: vec3(100.5, -200.25, 30.0),
        linear_velocity: TrVec3::default(),
        angular_velocity: TrVec3::default(),
    };
    assert_eq!(
        Archetype::Ball(ball_spawn(&ball)).to_archetype_string(),
        concat!(
            "{\"ObjectArchetype\":\"Archetypes.Ball.Ball_GameEditor\",",
            "\"StartLocationX\":100.5000,\"StartLocationY\":-200.2500,",
            "\"StartLocationZ\":30.0000,",
            "\"VelocityStartRotationP\":0,\"VelocityStartRotationY\":0,",
            "\"VelocityStartRotationR\":0,\"VelocityStartSpeed\":0.0000}"
        )
    );
}

/// Decodes the corpus fixture's velocity rotator (P=8492, Y=-15670,
/// Speed=1554.7803) back into a velocity vector, then checks that the
/// builder's rotator+speed encoding reproduces it.
#[test]
fn velocity_rotator_round_trips_corpus_fixture_values() {
    const UNITS_TO_RADIANS: f64 = std::f64::consts::PI / 32768.0;
    let pitch_radians = 8492.0 * UNITS_TO_RADIANS;
    let yaw_radians = -15670.0 * UNITS_TO_RADIANS;
    let speed = 1554.7803f64;
    let velocity = vec3(
        (speed * pitch_radians.cos() * yaw_radians.cos()) as f32,
        (speed * pitch_radians.cos() * yaw_radians.sin()) as f32,
        (speed * pitch_radians.sin()) as f32,
    );

    let (pitch, yaw, roll, encoded_speed) = velocity_rotator_and_speed(velocity);
    assert!((pitch - 8492).abs() <= 1, "pitch was {pitch}");
    assert!((yaw - -15670).abs() <= 1, "yaw was {yaw}");
    assert_eq!(roll, 0);
    assert!(
        (f64::from(encoded_speed) - speed).abs() < 0.01,
        "speed was {encoded_speed}"
    );
    assert_eq!(format!("{encoded_speed:.4}"), "1554.7803");
}

#[test]
fn ball_archetype_formats_fixture_location_exactly() {
    // Location taken from a decoded real pack; formatting must reproduce
    // the game's four-decimal fixed notation byte-for-byte.
    let ball = TrBallState {
        location: vec3(24.5048, 4269.2217, 224.4333),
        linear_velocity: TrVec3::default(),
        angular_velocity: TrVec3::default(),
    };
    let archetype = Archetype::Ball(ball_spawn(&ball)).to_archetype_string();
    assert!(
        archetype.contains(
            "\"StartLocationX\":24.5048,\"StartLocationY\":4269.2217,\"StartLocationZ\":224.4333"
        ),
        "archetype was {archetype}"
    );
}

/// The spawn point must carry the captured car's transform — the game
/// places the training car from this entry (a default-placed spawn point
/// was the "car always spawns at center field" bug).
#[test]
fn spawn_point_archetype_carries_the_captured_car_transform() {
    let car = TrCarState {
        location: vec3(-599.9999, -700.0001, 530.0),
        rotation: TrRotator {
            pitch: -837,
            yaw: 3634,
            roll: 128,
        },
        is_primary: 1,
        ..TrCarState::default()
    };
    assert_eq!(
        Archetype::CarSpawnPoint(car_spawn_point(&car, false)).to_archetype_string(),
        concat!(
            "{\"ObjectArchetype\":\"Archetypes.GameEditor.DynamicSpawnPointMesh\",",
            "\"LocationX\":-599.9999,\"LocationY\":-700.0001,\"LocationZ\":530.0000,",
            "\"RotationP\":-837,\"RotationY\":3634,\"RotationR\":128,",
            "\"VelocityStartSpeed\":0.0000}"
        )
    );
}

/// Ground-clipped captures (Z below a resting car's ~17uu origin) are
/// clamped up to the floor so the spawn is not embedded in the ground;
/// anything at or above the floor — including aerial captures — passes
/// through untouched.
#[test]
fn spawn_point_clamps_ground_clipping_z_but_passes_airborne_z_through() {
    let clipping = TrCarState {
        location: vec3(100.0, 200.0, 12.5),
        ..TrCarState::default()
    };
    let clipped = car_spawn_point(&clipping, false);
    assert_eq!(clipped.location_z, MIN_SPAWN_LOCATION_Z);
    assert!(
        Archetype::CarSpawnPoint(clipped)
            .to_archetype_string()
            .contains("\"LocationZ\":17.0000"),
    );

    let aerial = TrCarState {
        location: vec3(100.0, 200.0, 1234.5),
        ..TrCarState::default()
    };
    assert_eq!(car_spawn_point(&aerial, false).location_z, 1234.5);
}

#[test]
fn car_archetype_matches_corpus_shape() {
    let car = TrCarState {
        location: vec3(-599.9999, -700.0001, 530.0),
        rotation: TrRotator {
            pitch: -837,
            yaw: 3634,
            roll: 0,
        },
        is_primary: 1,
        ..TrCarState::default()
    };
    assert_eq!(
        Archetype::PlayerCar(player_car_spawn(&car)).to_archetype_string(),
        concat!(
            "{\"IsPC\":true,",
            "\"LocationX\":-599.9999,\"LocationY\":-700.0001,\"LocationZ\":530.0000,",
            "\"RotationP\":-837,\"RotationY\":3634,\"RotationR\":0}"
        )
    );
}

/// The emitted strings must parse back into the same archetype kinds and
/// re-serialize byte-identically (stable through a parse/write round
/// trip). Numeric-value equality is only up to `%.4f` rounding, so the
/// assertion is on the regenerated strings.
#[test]
fn built_archetype_strings_parse_back_to_typed_values() {
    let ball = TrBallState {
        location: vec3(24.5, 4269.25, 224.4375),
        linear_velocity: vec3(500.0, 0.0, 250.0),
        angular_velocity: TrVec3::default(),
    };
    let car = TrCarState {
        location: vec3(-600.0, -700.0, 17.0),
        rotation: TrRotator {
            pitch: -837,
            yaw: 3634,
            roll: 0,
        },
        is_primary: 1,
        ..TrCarState::default()
    };
    let strings = build_round_archetypes(&ball, &[car], false);
    let parsed: Vec<Archetype> = strings
        .iter()
        .map(|string| Archetype::parse(string))
        .collect();
    assert!(matches!(parsed[0], Archetype::Ball(_)));
    assert!(matches!(parsed[1], Archetype::CarSpawnPoint(_)));
    assert!(matches!(parsed[2], Archetype::PlayerCar(_)));
    for (archetype, string) in parsed.iter().zip(&strings) {
        assert_eq!(&archetype.to_archetype_string(), string);
    }
}

#[test]
fn round_archetypes_emit_ball_spawn_and_single_primary_car() {
    let ball = TrBallState::default();
    let secondary = TrCarState {
        location: vec3(1.0, 2.0, 3.0),
        ..TrCarState::default()
    };
    let primary = TrCarState {
        location: vec3(-100.0, 200.0, 17.0),
        is_primary: 1,
        ..TrCarState::default()
    };
    let archetypes = build_round_archetypes(&ball, &[secondary, primary], false);
    assert_eq!(archetypes.len(), 3);
    assert!(archetypes[0].contains("Ball_GameEditor"));
    // The spawn point tracks the primary car (the game places the training
    // car from it), not the secondary car or the editor default.
    assert!(archetypes[1].contains("DynamicSpawnPointMesh"));
    assert!(archetypes[1].contains("\"LocationX\":-100.0000,\"LocationY\":200.0000"));
    // The primary-flagged car wins even when it is not first.
    assert!(archetypes[2].starts_with("{\"IsPC\":true,"));
    assert!(archetypes[2].contains("\"LocationX\":-100.0000"));
}

#[test]
fn round_archetypes_without_cars_fall_back_to_default_car() {
    let archetypes = build_round_archetypes(&TrBallState::default(), &[], false);
    assert_eq!(archetypes.len(), 3);
    assert!(archetypes[2].starts_with("{\"IsPC\":true,"));
}

/// Forward-speed projection for momentum capture: only the component of
/// the captured velocity along the car's facing survives.
#[test]
fn forward_speed_projects_velocity_onto_facing() {
    // Facing +Y (yaw = 16384 units = 90 degrees), moving straight +Y.
    let forward = TrCarState {
        rotation: TrRotator {
            pitch: 0,
            yaw: 16384,
            roll: 0,
        },
        linear_velocity: vec3(0.0, 1200.0, 0.0),
        ..TrCarState::default()
    };
    assert!(
        (forward_speed(&forward) - 1200.0).abs() < 1e-6,
        "forward speed was {}",
        forward_speed(&forward)
    );

    // Pure sideways drift (facing +X, moving +Y): projection ~0 -> 0.0.
    let sideways = TrCarState {
        linear_velocity: vec3(0.0, 800.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(forward_speed(&sideways), 0.0);

    // Reversing (facing +X, moving -X): a negative projection clamps to 0.
    let reversing = TrCarState {
        linear_velocity: vec3(-500.0, 0.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(forward_speed(&reversing), 0.0);

    // Airborne: facing 45 degrees up along +X (pitch = 8192 units), moving
    // diagonally up-forward; the along-facing share is kept, including the
    // vertical part of the facing.
    let airborne = TrCarState {
        rotation: TrRotator {
            pitch: 8192,
            yaw: 0,
            roll: 0,
        },
        linear_velocity: vec3(300.0, 0.0, 300.0),
        ..TrCarState::default()
    };
    let expected = 600.0 / std::f64::consts::SQRT_2; // 300*cos45 + 300*sin45
    assert!(
        (forward_speed(&airborne) - expected).abs() < 1e-6,
        "airborne forward speed was {}",
        forward_speed(&airborne)
    );
}

/// Sub-1uu/s projections flush to an exact corpus-matching 0.0.
#[test]
fn forward_speed_flushes_physics_noise_to_zero() {
    let crawling = TrCarState {
        linear_velocity: vec3(0.5, 0.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(forward_speed(&crawling), 0.0);
}

/// Momentum capture lands in the serialized spawn-point string; the
/// toggle off keeps the corpus 0.0.
#[test]
fn spawn_point_momentum_toggle_controls_velocity_start_speed() {
    let car = TrCarState {
        rotation: TrRotator {
            pitch: 0,
            yaw: 16384,
            roll: 0,
        },
        linear_velocity: vec3(0.0, 987.6543, 0.0),
        is_primary: 1,
        ..TrCarState::default()
    };
    assert!(
        Archetype::CarSpawnPoint(car_spawn_point(&car, true))
            .to_archetype_string()
            .contains("\"VelocityStartSpeed\":987.6543"),
    );
    assert!(
        Archetype::CarSpawnPoint(car_spawn_point(&car, false))
            .to_archetype_string()
            .contains("\"VelocityStartSpeed\":0.0000"),
    );
}

/// A car driving where it points (fast, straight down its facing) is fully
/// representable: no momentum warning.
#[test]
fn momentum_warning_none_for_along_facing_motion() {
    let forward = TrCarState {
        rotation: TrRotator {
            pitch: 0,
            yaw: 16384,
            roll: 0,
        },
        linear_velocity: vec3(0.0, 1400.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(momentum_warning(&forward), None);
}

/// Below the total-speed floor no warning fires, however unrepresentable
/// the direction: a slow car loses little absolute momentum.
#[test]
fn momentum_warning_none_below_min_speed() {
    // Pure sideways drift, but under MOMENTUM_WARNING_MIN_SPEED.
    let slow_drift = TrCarState {
        linear_velocity: vec3(0.0, 250.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(momentum_warning(&slow_drift), None);
}

/// A fast lateral drift (facing +X, moving +Y) loses everything to the
/// projection: warns with the documented wording and numbers.
#[test]
fn momentum_warning_fires_for_fast_sideways_drift() {
    let drift = TrCarState {
        linear_velocity: vec3(0.0, 900.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(
        momentum_warning(&drift).as_deref(),
        Some(
            "car moving 900 uu/s at 90\u{b0} off facing; only 0 uu/s representable as spawn momentum"
        )
    );
}

/// Reversing (velocity opposed to facing) clamps the emitted speed to 0,
/// so the whole magnitude is lost and the angle exceeds 90 degrees.
#[test]
fn momentum_warning_fires_for_reversing_car() {
    let reversing = TrCarState {
        linear_velocity: vec3(-1000.0, 0.0, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(
        momentum_warning(&reversing).as_deref(),
        Some(
            "car moving 1000 uu/s at 180\u{b0} off facing; only 0 uu/s representable as spawn momentum"
        )
    );
}

/// The angle gate catches a fast slide even when the lost magnitude stays
/// under the lost-speed threshold, and a mild off-axis angle under both
/// thresholds stays quiet.
#[test]
fn momentum_warning_angle_gate_and_quiet_zone() {
    // 45 degrees off facing at 500 uu/s: lost = 500*sin(45) ~ 354 <= 400,
    // but the angle (45 > 30) trips the warning.
    let sliding = TrCarState {
        linear_velocity: vec3(353.5534, 353.5534, 0.0),
        ..TrCarState::default()
    };
    assert_eq!(
        momentum_warning(&sliding).as_deref(),
        Some(
            "car moving 500 uu/s at 45\u{b0} off facing; only 354 uu/s representable as spawn momentum"
        )
    );

    // 20 degrees off at 800 uu/s: lost = 800*sin(20) ~ 274 <= 400 and the
    // angle is under 30, so the capture is representable enough.
    let angle = 20.0_f64.to_radians();
    let mild = TrCarState {
        linear_velocity: vec3(
            (800.0 * angle.cos()) as f32,
            (800.0 * angle.sin()) as f32,
            0.0,
        ),
        ..TrCarState::default()
    };
    assert_eq!(momentum_warning(&mild), None);
}
