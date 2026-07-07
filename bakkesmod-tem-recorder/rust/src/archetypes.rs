//! Serialized-archetype string builders for training-pack rounds.
//!
//! TEMPORARY: replace with the typed `BallSpawn`/`CarSpawn` constructors
//! from `subtr-actor-training` when they land (phase-3). Until then this
//! module is the ONLY place archetype strings are constructed, hand-rolled
//! to byte-match the strings the in-game training editor writes, as
//! observed across a corpus of decoded real packs:
//!
//! * fixed key order per archetype kind,
//! * floats printed with exactly four decimal places (`24.5048`,
//!   `-599.9999`, `0.0000`),
//! * rotations as plain integers in Unreal rotator units
//!   (65536 = full turn),
//! * ball initial velocity stored as a direction rotator plus a speed
//!   magnitude, not a vector.

use crate::abi::{TrBallState, TrCarState, TrVec3};

/// Unreal rotator units per radian: 65536 units per full turn.
const UNITS_PER_RADIAN: f64 = 32768.0 / std::f64::consts::PI;

/// Formats a float the way the game's archetype writer does: fixed four
/// decimal places, no exponent.
fn fmt4(value: f32) -> String {
    format!("{value:.4}")
}

/// Converts an angle in radians to integer Unreal rotator units.
fn rotator_units(radians: f64) -> i32 {
    (radians * UNITS_PER_RADIAN).round() as i32
}

/// Decomposes a velocity vector into (pitch, yaw, roll) Unreal rotator
/// units plus a speed magnitude, matching the ball archetype's
/// `VelocityStartRotation{P,Y,R}` + `VelocityStartSpeed` encoding.
///
/// Roll is meaningless for a direction and is emitted as 0; corpus packs
/// written by the in-game editor show small nonzero rolls.
/// TODO(phase-3): confirm roll handling against the typed constructors.
pub fn velocity_rotator_and_speed(velocity: TrVec3) -> (i32, i32, i32, f32) {
    let x = f64::from(velocity.x);
    let y = f64::from(velocity.y);
    let z = f64::from(velocity.z);
    let horizontal = x.hypot(y);
    let speed = (horizontal * horizontal + z * z).sqrt();
    if speed == 0.0 {
        return (0, 0, 0, 0.0);
    }
    let pitch = z.atan2(horizontal);
    let yaw = y.atan2(x);
    (rotator_units(pitch), rotator_units(yaw), 0, speed as f32)
}

/// Builds the ball archetype string, e.g.
/// `{"ObjectArchetype":"Archetypes.Ball.Ball_GameEditor","StartLocationX":24.5048,...,"VelocityStartSpeed":1554.7803}`.
pub fn ball_archetype(ball: &TrBallState) -> String {
    let (pitch, yaw, roll, speed) = velocity_rotator_and_speed(ball.linear_velocity);
    // TODO(phase-3): ball angular velocity is not representable in the
    // current archetype format and is dropped here.
    format!(
        concat!(
            "{{\"ObjectArchetype\":\"Archetypes.Ball.Ball_GameEditor\",",
            "\"StartLocationX\":{},\"StartLocationY\":{},\"StartLocationZ\":{},",
            "\"VelocityStartRotationP\":{},\"VelocityStartRotationY\":{},",
            "\"VelocityStartRotationR\":{},\"VelocityStartSpeed\":{}}}"
        ),
        fmt4(ball.location.x),
        fmt4(ball.location.y),
        fmt4(ball.location.z),
        pitch,
        yaw,
        roll,
        fmt4(speed),
    )
}

/// Builds the spawn-point marker archetype the in-game editor writes into
/// every round, at its default center-field placement.
///
/// TODO(phase-3): corpus packs always contain exactly one of these; whether
/// it is required for the pack to load, and whether it should track the
/// primary car instead of the default placement, needs confirmation against
/// the typed constructors.
pub fn spawn_point_archetype() -> String {
    concat!(
        "{\"ObjectArchetype\":\"Archetypes.GameEditor.DynamicSpawnPointMesh\",",
        "\"LocationX\":0.0000,\"LocationY\":0.0000,\"LocationZ\":30.0000,",
        "\"RotationP\":0,\"RotationY\":16384,\"RotationR\":0,",
        "\"VelocityStartSpeed\":0.0000}"
    )
    .to_string()
}

/// Builds the player-car archetype string, e.g.
/// `{"IsPC":true,"LocationX":-599.9999,...,"RotationR":0}`.
pub fn car_archetype(car: &TrCarState) -> String {
    // TODO(phase-3): car linear/angular velocity and boost are not
    // representable in the current archetype format and are dropped here.
    format!(
        concat!(
            "{{\"IsPC\":true,",
            "\"LocationX\":{},\"LocationY\":{},\"LocationZ\":{},",
            "\"RotationP\":{},\"RotationY\":{},\"RotationR\":{}}}"
        ),
        fmt4(car.location.x),
        fmt4(car.location.y),
        fmt4(car.location.z),
        car.rotation.pitch,
        car.rotation.yaw,
        car.rotation.roll,
    )
}

/// Builds the full `SerializedArchetypes` list for one captured shot, in
/// the order every corpus round uses: ball, spawn-point marker, then
/// exactly one `"IsPC":true` player car.
///
/// The emitted car is the captured car flagged `is_primary` (falling back
/// to the first captured car, or a default car at the origin when none were
/// captured). The corpus never contains a second car, so additional
/// captured cars are dropped rather than emitted with an unobserved
/// `"IsPC":false` vocabulary.
/// TODO(phase-3): revisit multi-car emission once the typed constructors
/// pin down the real vocabulary.
pub fn build_round_archetypes(ball: &TrBallState, cars: &[TrCarState]) -> Vec<String> {
    let primary = cars
        .iter()
        .find(|car| car.is_primary != 0)
        .or_else(|| cars.first())
        .copied()
        .unwrap_or_default();
    vec![
        ball_archetype(ball),
        spawn_point_archetype(),
        car_archetype(&primary),
    ]
}

#[cfg(test)]
#[path = "archetypes_tests.rs"]
mod tests;
