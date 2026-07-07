//! `repr(C)` types shared with the C++ plugin.
//!
//! Every struct here is mirrored by a `typedef struct` in
//! `include/tem_recorder.h`; sizes and field offsets are locked by
//! `lib_tests.rs` so drift between the two is caught by `cargo test`.

/// A position or velocity vector in Unreal units, matching BakkesMod's
/// `Vector` (three floats).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A rotation in integer Unreal rotator units (65536 units = full turn,
/// 16384 = 90 degrees), matching BakkesMod's `Rotator` (three ints).
///
/// These pass through to the training-pack archetype `RotationP`/`Y`/`R`
/// fields unchanged.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrRotator {
    pub pitch: i32,
    pub yaw: i32,
    pub roll: i32,
}

/// Captured ball state for one shot.
///
/// The current `.tem` archetype format stores ball velocity as a direction
/// rotator plus a speed magnitude and has no angular-velocity field at all;
/// `angular_velocity` is carried through the ABI anyway so the plugin does
/// not need to change when the typed archetype constructors land
/// (TODO(phase-3)).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrBallState {
    pub location: TrVec3,
    pub linear_velocity: TrVec3,
    pub angular_velocity: TrVec3,
}

/// Captured car state for one shot.
///
/// The current `.tem` archetype format only stores car location and
/// rotation; `linear_velocity`, `angular_velocity`, and `boost_amount`
/// (0.0..=1.0 as BakkesMod reports it) are captured for phase-3 but are not
/// yet representable in the serialized pack.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrCarState {
    pub location: TrVec3,
    pub rotation: TrRotator,
    pub linear_velocity: TrVec3,
    pub angular_velocity: TrVec3,
    /// Boost fraction 0.0..=1.0 as reported by BakkesMod's `BoostWrapper`.
    pub boost_amount: f32,
    /// Nonzero for the car the shot is "for"; it becomes the `IsPC` car in
    /// the round archetypes.
    pub is_primary: u8,
    /// 0 or 1; captured for phase-3, not currently serialized.
    pub team: u8,
}

/// One captured shot: the ball plus every car on the field at the captured
/// replay frame.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrCapturedShot {
    pub ball: TrBallState,
    /// Round time limit in seconds.
    pub time_limit: f32,
    /// Pointer to `car_count` contiguous `TrCarState` values; may be null
    /// when `car_count` is zero.
    pub cars: *const TrCarState,
    pub car_count: usize,
}
