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

/// Forward speeds below this (in uu/s) are written as an exact `0.0`: a
/// car that is standing still in the replay should produce the corpus'
/// stationary spawn, not a fraction-of-a-uu crawl from physics noise.
pub const MIN_FORWARD_SPEED: f64 = 1.0;

/// The signed projection of a captured car's velocity onto its facing
/// (negative when reversing) plus the total speed, both in uu/s. Shared by
/// [`forward_speed`] and [`momentum_warning`].
fn facing_projection_and_speed(car: &TrCarState) -> (f64, f64) {
    const UNITS_TO_RADIANS: f64 = std::f64::consts::PI / 32768.0;
    let pitch = f64::from(car.rotation.pitch) * UNITS_TO_RADIANS;
    let yaw = f64::from(car.rotation.yaw) * UNITS_TO_RADIANS;
    // Unreal forward vector for a (pitch, yaw) facing; roll does not move
    // the forward axis.
    let forward = (
        pitch.cos() * yaw.cos(),
        pitch.cos() * yaw.sin(),
        pitch.sin(),
    );
    let velocity = car.linear_velocity;
    let x = f64::from(velocity.x);
    let y = f64::from(velocity.y);
    let z = f64::from(velocity.z);
    let projection = x * forward.0 + y * forward.1 + z * forward.2;
    let speed = (x * x + y * y + z * z).sqrt();
    (projection, speed)
}

/// The forward component of a captured car's velocity along its facing, in
/// uu/s, for the spawn mesh's `VelocityStartSpeed` field.
///
/// The spawn mesh is the editor's car-start-speed feature, and — per the
/// game's own class dump (RLSDK `TAGame.DynamicSpawnPointMesh_TA`) — the
/// class carries ONLY `VelocityStartSpeed: float` and `MaxStartSpeed:
/// float`. There is no direction property (unlike the ball's
/// `Ball_GameEditor_TA`, which has a full `VelocityStartRotation: FRotator`
/// next to its `VelocityStartSpeed`), so car spawn momentum is
/// scalar-along-facing BY GAME DESIGN: a full velocity vector is not
/// representable, and projecting the captured velocity onto the car's
/// facing is the maximal-fidelity encoding the format admits. Lateral
/// drift and any motion opposed to the facing are dropped (see
/// [`momentum_warning`] for the capture-time diagnostic):
///
/// * projection < 0 (reversing / moving away from the facing) clamps to
///   `0.0` — a negative scalar would be interpreted as forward speed by an
///   unknown convention, and a spawned car cannot start in reverse;
/// * projection below [`MIN_FORWARD_SPEED`] flushes to exactly `0.0`.
///
/// The facing includes pitch, so an airborne car keeps the along-facing
/// share of its momentum.
///
/// Starting boost is NOT supported by the format: the class dump shows no
/// starting-boost property on either editor class
/// (`Ball_GameEditor_TA` / `DynamicSpawnPointMesh_TA`), so the captured
/// `boost_amount` is carried across the ABI but cannot land in the pack.
///
/// TODO(in-game): `MaxStartSpeed` exists on `DynamicSpawnPointMesh_TA` and
/// MAY clamp large written speeds; check whether captured speeds above the
/// editor slider max survive a load/play round-trip (we do not emit
/// `MaxStartSpeed` yet).
pub fn forward_speed(car: &TrCarState) -> f64 {
    let (projection, _speed) = facing_projection_and_speed(car);
    if projection < MIN_FORWARD_SPEED {
        0.0
    } else {
        projection
    }
}

/// Default total speed below which no momentum warning is raised, in uu/s.
/// A slow-moving car loses little absolute momentum whatever its direction,
/// and near-stationary velocity direction is mostly physics noise. The
/// plugin exposes this as the `replay_to_training_momentum_warn_min_speed`
/// cvar; setting it very high effectively disables the warning entirely.
pub const MOMENTUM_WARNING_MIN_SPEED: f64 = 300.0;

/// Default unrepresentable (lost) velocity magnitude above which a momentum
/// warning is raised, in uu/s. Roughly a third of max driving speed:
/// enough loss to visibly change how a recreated shot plays. The plugin
/// exposes this as the `replay_to_training_momentum_warn_min_lost` cvar.
pub const MOMENTUM_WARNING_MIN_LOST_SPEED: f64 = 400.0;

/// Default angle between velocity and facing above which a momentum warning
/// is raised, in degrees. Past ~20° the car is drifting/sliding rather than
/// driving where it points, so the along-facing projection misrepresents
/// the motion even when the lost magnitude is modest. At 20° the sideways
/// component is ~34% of the car's speed while the forward projection only
/// loses ~6% — a modest speed hit but a clearly off-axis motion, so the
/// gate defaults tight here per user feedback. The plugin exposes this as
/// the `replay_to_training_momentum_warn_max_angle` cvar.
pub const MOMENTUM_WARNING_MAX_OFF_AXIS_DEGREES: f64 = 20.0;

/// Capture-time diagnostic for momentum capture: a human-readable warning
/// when a meaningful share of the car's velocity cannot be encoded in the
/// spawn mesh's scalar `VelocityStartSpeed` (see [`forward_speed`] for why
/// the format only stores speed-along-facing).
///
/// The three thresholds are supplied by the caller (the plugin reads them
/// from persisted cvars; [`momentum_warning`] wraps this with the crate
/// defaults for tests and any caller that does not tune them). With total
/// speed `s`, emitted forward speed `f` ([`forward_speed`], so already
/// clamped/flushed) and signed facing projection `p`, the lost magnitude is
/// `sqrt(max(s² − f², 0))` (reversing ⇒ everything lost) and the off-axis
/// angle is `acos(clamp(p / s, −1, 1))`. Warns — capture still proceeds —
/// when `s > min_speed` AND (lost `> min_lost_speed` OR angle `>
/// max_off_axis_degrees`). Returns `None` when the velocity is
/// representable enough.
pub fn momentum_warning_with_thresholds(
    car: &TrCarState,
    min_speed: f64,
    min_lost_speed: f64,
    max_off_axis_degrees: f64,
) -> Option<String> {
    let (projection, speed) = facing_projection_and_speed(car);
    if speed <= min_speed {
        return None;
    }
    let representable = forward_speed(car);
    let lost = (speed * speed - representable * representable)
        .max(0.0)
        .sqrt();
    let off_axis_degrees = (projection / speed).clamp(-1.0, 1.0).acos().to_degrees();
    if lost <= min_lost_speed && off_axis_degrees <= max_off_axis_degrees {
        return None;
    }
    Some(format!(
        "car moving {speed:.0} uu/s at {off_axis_degrees:.0}\u{b0} off facing; \
         only {representable:.0} uu/s representable as spawn momentum"
    ))
}

/// [`momentum_warning_with_thresholds`] using the crate default thresholds
/// ([`MOMENTUM_WARNING_MIN_SPEED`], [`MOMENTUM_WARNING_MIN_LOST_SPEED`],
/// [`MOMENTUM_WARNING_MAX_OFF_AXIS_DEGREES`]).
pub fn momentum_warning(car: &TrCarState) -> Option<String> {
    momentum_warning_with_thresholds(
        car,
        MOMENTUM_WARNING_MIN_SPEED,
        MOMENTUM_WARNING_MIN_LOST_SPEED,
        MOMENTUM_WARNING_MAX_OFF_AXIS_DEGREES,
    )
}

/// Builds the spawn-point (`DynamicSpawnPointMesh`) archetype for the
/// captured primary car.
///
/// In-game testing showed the game places the training car from THIS entry,
/// not from the `IsPC` player-car entry (Psyonix packs have all-zero `IsPC`
/// transforms and still place correctly), so the captured transform must
/// land here. Location and rotation pass through faithfully apart from the
/// [`MIN_SPAWN_LOCATION_Z`] clamp. With `capture_momentum`,
/// `VelocityStartSpeed` carries the car's [`forward_speed`]; otherwise it
/// stays `0.0` like every corpus spawn point.
pub fn car_spawn_point(car: &TrCarState, capture_momentum: bool) -> CarSpawn {
    CarSpawn {
        location_x: f64::from(car.location.x),
        location_y: f64::from(car.location.y),
        location_z: f64::from(car.location.z).max(MIN_SPAWN_LOCATION_Z),
        rotation_p: car.rotation.pitch,
        rotation_y: car.rotation.yaw,
        rotation_r: car.rotation.roll,
        velocity_start_speed: Some(if capture_momentum {
            forward_speed(car)
        } else {
            0.0
        }),
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
///
/// With `capture_momentum`, the spawn point carries the primary car's
/// [`forward_speed`] (see [`car_spawn_point`]).
pub fn build_round_archetypes(
    ball: &TrBallState,
    cars: &[TrCarState],
    capture_momentum: bool,
) -> Vec<String> {
    let primary = cars
        .iter()
        .find(|car| car.is_primary != 0)
        .or_else(|| cars.first())
        .copied()
        .unwrap_or_default();
    vec![
        Archetype::Ball(ball_spawn(ball)).to_archetype_string(),
        Archetype::CarSpawnPoint(car_spawn_point(&primary, capture_momentum)).to_archetype_string(),
        Archetype::PlayerCar(player_car_spawn(&primary)).to_archetype_string(),
    ]
}

#[cfg(test)]
#[path = "archetypes_tests.rs"]
mod tests;
