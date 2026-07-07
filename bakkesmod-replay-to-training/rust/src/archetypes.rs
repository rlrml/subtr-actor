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

/// Minimum spawn-point height in uu. A resting car's origin sits at ~17uu
/// (half the hitbox height plus wheel clearance); replay physics can sample
/// a car a hair below that mid-landing, and a spawn point clipped into the
/// floor produces a broken spawn in the editor. Captured Z is clamped up to
/// this floor; anything above it (aerial captures) passes through untouched
/// because mid-air captures are legitimate aerial scenarios.
pub const MIN_SPAWN_LOCATION_Z: f64 = 17.0;

/// Builds the spawn-point (`DynamicSpawnPointMesh`) archetype for the
/// captured primary car.
///
/// In-game testing showed the game places the training car from THIS entry,
/// not from the `IsPC` player-car entry (Psyonix packs have all-zero `IsPC`
/// transforms and still place correctly), so the captured transform must
/// land here. Location and rotation pass through faithfully apart from the
/// [`MIN_SPAWN_LOCATION_Z`] clamp; `VelocityStartSpeed` stays 0 like every
/// corpus spawn point (car velocity is not representable in the format).
pub fn car_spawn_point(car: &TrCarState) -> CarSpawn {
    CarSpawn {
        location_x: f64::from(car.location.x),
        location_y: f64::from(car.location.y),
        location_z: f64::from(car.location.z).max(MIN_SPAWN_LOCATION_Z),
        rotation_p: car.rotation.pitch,
        rotation_y: car.rotation.yaw,
        rotation_r: car.rotation.roll,
        ..CarSpawn::default()
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
/// the order every corpus round uses: ball, spawn-point marker, then
/// exactly one `"IsPC":true` player car.
///
/// The spawn-point marker carries the captured primary car's transform
/// (see [`car_spawn_point`]): in-game testing confirmed the game spawns the
/// training car from the `DynamicSpawnPointMesh` entry, so emitting it at
/// the editor default put every captured shot at center field. The `IsPC`
/// entry keeps the same captured transform it always carried — Psyonix
/// packs zero it, but the game demonstrably ignores it for placement, and
/// keeping it preserves information for round-trip tooling.
///
/// The emitted car is the captured car flagged `is_primary` (falling back
/// to the first captured car, or a default car at the origin when none were
/// captured). The corpus never contains a second car, so additional
/// captured cars are dropped rather than emitted with an unobserved
/// `"IsPC":false` vocabulary.
///
/// TODO(in-game): revisit multi-car emission if in-game testing shows the
/// editor accepts more than one car per round.
pub fn build_round_archetypes(ball: &TrBallState, cars: &[TrCarState]) -> Vec<String> {
    let primary = cars
        .iter()
        .find(|car| car.is_primary != 0)
        .or_else(|| cars.first())
        .copied()
        .unwrap_or_default();
    vec![
        Archetype::Ball(ball_spawn(ball)).to_archetype_string(),
        Archetype::CarSpawnPoint(car_spawn_point(&primary)).to_archetype_string(),
        Archetype::PlayerCar(player_car_spawn(&primary)).to_archetype_string(),
    ]
}

#[cfg(test)]
#[path = "archetypes_tests.rs"]
mod tests;
