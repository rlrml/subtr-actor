//! `repr(C)` types shared with the C++ plugin.
//!
//! Every struct here is mirrored by a `typedef struct` in
//! `include/replay_to_training.h`; sizes and field offsets are locked by
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
/// The `.tem` archetype format stores ball velocity as a direction rotator
/// plus a speed magnitude and has no angular-velocity field at all
/// (confirmed by the typed `subtr_actor_training::BallSpawn`);
/// `angular_velocity` is carried through the ABI anyway so the plugin does
/// not need to change if the format ever grows one.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrBallState {
    pub location: TrVec3,
    pub linear_velocity: TrVec3,
    pub angular_velocity: TrVec3,
}

/// Captured car state for one shot.
///
/// The `.tem` archetype format only stores car location and rotation
/// (confirmed by the typed `subtr_actor_training::PlayerCarSpawn`);
/// `linear_velocity`, `angular_velocity`, and `boost_amount` (0.0..=1.0 as
/// BakkesMod reports it) are captured across the ABI but are not
/// representable in the serialized pack.
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
    /// 0 or 1; captured across the ABI, not representable in the
    /// serialized pack.
    pub team: u8,
}

/// Capture mode discriminant carried in [`TrCapturedShot::mode`]: `0` for
/// an offensive (striker/shot) capture, `1` for a defensive (goalie/save)
/// capture. Mirrors `crate::mirror::CaptureMode`.
pub const TR_CAPTURE_MODE_SHOT: u8 = 0;
/// See [`TR_CAPTURE_MODE_SHOT`].
pub const TR_CAPTURE_MODE_SAVE: u8 = 1;

/// One captured shot: the ball plus every car on the field at the captured
/// replay frame, and the per-capture options the plugin's cvars selected.
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
    /// Capture mode: [`TR_CAPTURE_MODE_SHOT`] (offensive) or
    /// [`TR_CAPTURE_MODE_SAVE`] (defensive). Decides the pack training
    /// type assigned by the first capture and the orientation convention
    /// used by mirroring.
    pub mode: u8,
    /// Nonzero to auto-mirror the whole scenario 180° about field center
    /// when the captured primary car's team does not match the training
    /// convention for `mode` (cvar `replay_to_training_mirror_by_team`).
    pub mirror_by_team: u8,
    /// Nonzero to write the primary car's forward speed into the spawn
    /// mesh's `VelocityStartSpeed` (cvar
    /// `replay_to_training_capture_momentum`).
    pub capture_momentum: u8,
}
