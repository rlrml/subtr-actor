//! Auto-mirroring of captured scenarios into the training-pack field frame.
//!
//! Training scenarios live in a FIXED field frame, but replay captures come
//! from either team's perspective, so a capture taken from the "wrong" team
//! aims at the wrong goal. This module decides whether a capture needs to be
//! flipped ([`should_mirror`]) and performs the flip ([`mirror_shot`]): a
//! 180-degree rotation of the whole scenario about the vertical axis through
//! field center.
//!
//! # The striker convention, derived from the corpus
//!
//! The Psyonix-authored striker pack in the decoded corpus (`Temp(1)`,
//! "Diamond Pack May 2023", `Training_Striker`, 9 rounds) pins down which
//! goal striker scenarios attack:
//!
//! * Ball `StartLocationY`: mean ≈ +1870uu; 7 of 9 rounds are in the +Y
//!   half; the deepest spawn is +4502.2uu — within ~620uu of the +Y goal
//!   line at Y = 5120 — while the ball never spawns closer to the −Y goal
//!   than −3169.3uu.
//! * The car spawn sits on the −Y side of the ball (car `LocationY` < ball
//!   `StartLocationY`) in 8 of 9 rounds: the player is set up BEHIND the
//!   ball, shooting toward +Y.
//! * The near-goal serves make it unambiguous: rounds with ball Y at
//!   +3455.6 and +4502.2 have `VelocityStartRotationY` of +16998 and
//!   +16241 rotator units (≈ +93.4° and +89.2°, i.e. straight at +Y), whose
//!   velocity Y-components (+2778.6 and +2732.2 uu/s at speeds 2783.5 and
//!   2734.3) carry the serve INTO the +Y goal mouth.
//!
//! So: **striker scenarios attack the +Y goal**, with the training player
//! playing from the −Y half.
//!
//! # Team assumption and the resulting rule
//!
//! Replays follow the standard field convention: **blue / team 0 defends
//! the −Y goal and attacks +Y**; orange / team 1 is the reverse. Combining
//! that with the corpus convention above, the training player is oriented
//! exactly like a blue-team player:
//!
//! * **Shot (striker) capture** — the scenario must drive the ball at the
//!   +Y goal. A blue attacker already does; an orange attacker's shot aims
//!   at −Y and must be mirrored.
//! * **Save (goalie) capture** — no goalie pack exists in the corpus, so we
//!   make the natural choice that the training player occupies the SAME
//!   field end in both modes (flagged for in-game validation in the
//!   README): the defender defends the −Y goal, with the incoming ball
//!   heading toward −Y. A blue defender already matches; an orange defender
//!   (defending +Y) must be mirrored.
//!
//! Both modes therefore reduce to the same test: mirror exactly when the
//! captured primary car is on team 1 (orange).

use crate::abi::{TR_CAPTURE_MODE_SAVE, TrBallState, TrCarState, TrVec3};

/// Which training-pack semantic a capture is for. Decides the pack
/// [`subtr_actor_training::TrainingType`] assigned by the first capture and
/// the orientation convention mirroring enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Offensive capture (`replay_to_training_capture_shot`): the scenario
    /// player is a striker attacking the +Y goal.
    Shot,
    /// Defensive capture (`replay_to_training_capture_save`): the scenario
    /// player is a goalie defending the −Y goal.
    Save,
}

impl CaptureMode {
    /// Decodes the ABI `mode` byte (see `TrCapturedShot::mode`); unknown
    /// values fall back to [`CaptureMode::Shot`], the pre-existing behavior.
    pub fn from_abi(mode: u8) -> CaptureMode {
        match mode {
            TR_CAPTURE_MODE_SAVE => CaptureMode::Save,
            // TR_CAPTURE_MODE_SHOT and anything unknown.
            _ => CaptureMode::Shot,
        }
    }
}

/// Half a turn in Unreal rotator units (65536 units = full turn).
const HALF_TURN_UNITS: i64 = 32768;

/// Whether a captured scenario must be mirrored 180° so its orientation
/// matches the training convention for `mode`, given the captured primary
/// car's team (0 = blue, 1 = orange).
///
/// As derived in the module docs, the training player is oriented like a
/// blue-team player in BOTH modes (striker attacks +Y = blue's attacked
/// goal; goalie defends −Y = blue's defended goal), so the test collapses
/// to "was the captured player on orange?". The `mode` parameter is kept
/// so the derivation stays explicit at the call site and so a future
/// asymmetric convention (pending in-game goalie validation) only touches
/// this function.
pub fn should_mirror(mode: CaptureMode, primary_team: u8) -> bool {
    match mode {
        // Shot: the ball must head toward the +Y goal, which is the goal a
        // blue (team 0) player attacks. Orange captures aim at −Y: mirror.
        CaptureMode::Shot => primary_team == 1,
        // Save: the ball must head toward the defender's own goal at −Y,
        // which is the goal a blue (team 0) player defends. Orange captures
        // defend +Y: mirror.
        CaptureMode::Save => primary_team == 1,
    }
}

/// Adds half a turn to a yaw value with the game's wrapping u16 rotator
/// semantics: rotators are stored as `i32` across BakkesMod and the `.tem`
/// archetypes, but only their low 16 bits are meaningful (65536 units =
/// full turn), so the mirrored yaw is normalized into `0..=65535` rather
/// than allowed to walk off toward `i32::MAX` across repeated mirrors.
pub fn mirror_yaw(yaw: i32) -> i32 {
    (i64::from(yaw) + HALF_TURN_UNITS).rem_euclid(65536) as i32
}

/// Rotates a position or velocity vector 180° about the vertical axis
/// through field center: X and Y negate, Z is unchanged. For a velocity
/// this is exactly "yaw + half turn with pitch (and speed) unchanged" in
/// the archetype's rotator+speed encoding. Angular velocity transforms the
/// same way (a 180° yaw is a proper rotation, `det = +1`).
pub fn mirror_vec(value: TrVec3) -> TrVec3 {
    TrVec3 {
        x: -value.x,
        y: -value.y,
        z: value.z,
    }
}

/// Mirrors a captured ball state in place (see [`mirror_vec`]).
pub fn mirror_ball(ball: &mut TrBallState) {
    ball.location = mirror_vec(ball.location);
    ball.linear_velocity = mirror_vec(ball.linear_velocity);
    ball.angular_velocity = mirror_vec(ball.angular_velocity);
}

/// Mirrors a captured car state in place: location/velocities through
/// [`mirror_vec`], yaw through [`mirror_yaw`]; pitch and roll are
/// orientation components relative to the (yawed) body frame and are
/// unchanged by a rotation about the world Z axis.
pub fn mirror_car(car: &mut TrCarState) {
    car.location = mirror_vec(car.location);
    car.rotation.yaw = mirror_yaw(car.rotation.yaw);
    car.linear_velocity = mirror_vec(car.linear_velocity);
    car.angular_velocity = mirror_vec(car.angular_velocity);
}

/// Mirrors a WHOLE captured scenario (ball plus every car) 180° about
/// field center. Applied all-or-nothing so relative geometry between the
/// ball and all cars is preserved exactly.
pub fn mirror_shot(ball: &mut TrBallState, cars: &mut [TrCarState]) {
    mirror_ball(ball);
    for car in cars.iter_mut() {
        mirror_car(car);
    }
}

#[cfg(test)]
#[path = "mirror_tests.rs"]
mod tests;
