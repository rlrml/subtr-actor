//! Maps captured ABI state to the typed training-pack archetypes.
//!
//! Serialization (key order, `%.4f` float formatting, bare integer rotator
//! components) is owned by `subtr_actor_training::archetype`; this module
//! only converts the plugin's captured [`TrBallState`]/[`TrCarState`] values
//! into [`BallSpawn`]/[`PlayerCarSpawn`] structs — unit conversions such as
//! the velocity-vector → rotator+speed encoding are the plugin's own
//! concern.

use subtr_actor_training::{Archetype, BallSpawn, CarSpawn, PlayerCarSpawn};

use crate::abi::{TrBallState, TrCarState, TrVec3};

/// Unreal rotator units per radian: 65536 units per full turn.
const UNITS_PER_RADIAN: f64 = 32768.0 / std::f64::consts::PI;

/// Converts an angle in radians to integer Unreal rotator units.
fn rotator_units(radians: f64) -> i32 {
    (radians * UNITS_PER_RADIAN).round() as i32
}

/// Decomposes a velocity vector into (pitch, yaw, roll) Unreal rotator
/// units plus a speed magnitude, matching the ball archetype's
/// `VelocityStartRotation{P,Y,R}` + `VelocityStartSpeed` encoding.
///
/// Roll is meaningless for a direction and is emitted as 0, which matches
/// the corpus default ball ([`BallSpawn::default`] has
/// `velocity_start_rotation_r: 0`); packs written by the in-game editor
/// sometimes carry small nonzero rolls, but they do not affect the
/// direction.
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

/// Builds the typed ball archetype for a captured ball state.
///
/// Ball angular velocity is not representable in the archetype format
/// (confirmed against the typed [`BallSpawn`], which has no such field) and
/// is dropped.
pub fn ball_spawn(ball: &TrBallState) -> BallSpawn {
    let (pitch, yaw, roll, speed) = velocity_rotator_and_speed(ball.linear_velocity);
    BallSpawn {
        start_location_x: f64::from(ball.location.x),
        start_location_y: f64::from(ball.location.y),
        start_location_z: f64::from(ball.location.z),
        velocity_start_rotation_p: pitch,
        velocity_start_rotation_y: yaw,
        velocity_start_rotation_r: roll,
        velocity_start_speed: f64::from(speed),
        extras: Default::default(),
    }
}

/// Builds the typed player-car archetype for a captured car state.
///
/// Car linear/angular velocity and boost are not representable in the
/// archetype format (confirmed against the typed [`PlayerCarSpawn`], which
/// only models location and rotation) and are dropped.
pub fn player_car_spawn(car: &TrCarState) -> PlayerCarSpawn {
    PlayerCarSpawn {
        is_pc: true,
        location_x: Some(f64::from(car.location.x)),
        location_y: Some(f64::from(car.location.y)),
        location_z: Some(f64::from(car.location.z)),
        rotation_p: Some(car.rotation.pitch),
        rotation_y: Some(car.rotation.yaw),
        rotation_r: Some(car.rotation.roll),
        extras: Default::default(),
    }
}

/// Builds the full `SerializedArchetypes` list for one captured shot, in
/// the order every corpus round uses: ball, spawn-point marker (at the
/// editor's default placement, [`CarSpawn::default`]), then exactly one
/// `"IsPC":true` player car.
///
/// The emitted car is the captured car flagged `is_primary` (falling back
/// to the first captured car, or a default car at the origin when none were
/// captured). The corpus never contains a second car, so additional
/// captured cars are dropped rather than emitted with an unobserved
/// `"IsPC":false` vocabulary.
///
/// TODO(in-game): revisit multi-car emission if in-game testing shows the
/// editor accepts more than one car per round.
/// TODO(in-game): corpus packs always contain exactly one spawn-point
/// marker at the editor default; whether it is required for the pack to
/// load, and whether it should track the primary car instead, needs
/// in-game validation.
pub fn build_round_archetypes(ball: &TrBallState, cars: &[TrCarState]) -> Vec<String> {
    let primary = cars
        .iter()
        .find(|car| car.is_primary != 0)
        .or_else(|| cars.first())
        .copied()
        .unwrap_or_default();
    vec![
        Archetype::Ball(ball_spawn(ball)).to_archetype_string(),
        Archetype::CarSpawnPoint(CarSpawn::default()).to_archetype_string(),
        Archetype::PlayerCar(player_car_spawn(&primary)).to_archetype_string(),
    ]
}

#[cfg(test)]
#[path = "archetypes_tests.rs"]
mod tests;
