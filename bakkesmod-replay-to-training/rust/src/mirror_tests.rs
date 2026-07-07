use super::*;
use crate::abi::TrRotator;
use crate::archetypes::velocity_rotator_and_speed;

fn vec3(x: f32, y: f32, z: f32) -> TrVec3 {
    TrVec3 { x, y, z }
}

fn sample_ball() -> TrBallState {
    TrBallState {
        location: vec3(62.16, 4502.21, 776.38),
        linear_velocity: vec3(150.0, 2732.2, -90.0),
        angular_velocity: vec3(1.0, -2.0, 3.0),
    }
}

fn sample_car(team: u8) -> TrCarState {
    TrCarState {
        location: vec3(-600.0, 1767.9, 17.0),
        rotation: TrRotator {
            pitch: -837,
            yaw: 21707,
            roll: 128,
        },
        linear_velocity: vec3(400.0, 900.0, 12.0),
        angular_velocity: vec3(0.5, -0.25, 1.5),
        boost_amount: 0.33,
        is_primary: 1,
        team,
    }
}

#[test]
fn mirror_negates_locations_and_preserves_z() {
    let mut ball = sample_ball();
    let mut cars = [sample_car(1)];
    mirror_shot(&mut ball, &mut cars);
    assert_eq!(ball.location, vec3(-62.16, -4502.21, 776.38));
    assert_eq!(cars[0].location, vec3(600.0, -1767.9, 17.0));
    assert_eq!(cars[0].linear_velocity, vec3(-400.0, -900.0, 12.0));
    assert_eq!(cars[0].angular_velocity, vec3(-0.5, 0.25, 1.5));
}

#[test]
fn mirror_yaw_adds_half_turn_with_wrapping_u16_semantics() {
    // Plain case: no wrap needed.
    assert_eq!(mirror_yaw(0), 32768);
    assert_eq!(mirror_yaw(16384), 49152);
    // Wraps at the u16 boundary instead of walking past 65535.
    assert_eq!(mirror_yaw(32768), 0);
    assert_eq!(mirror_yaw(49152), 16384);
    assert_eq!(mirror_yaw(65535), 32767);
    // Negative yaws (BakkesMod rotators are signed) normalize into
    // 0..=65535: -16384 is the same angle as 49152, plus a half turn.
    assert_eq!(mirror_yaw(-16384), 16384);
    assert_eq!(mirror_yaw(-1), 32767);
    // Twice-mirrored is the identity angle (mod 65536).
    for yaw in [-32768, -16384, -1, 0, 3634, 21707, 65535] {
        assert_eq!(
            mirror_yaw(mirror_yaw(yaw)),
            (i64::from(yaw)).rem_euclid(65536) as i32,
            "double mirror of {yaw}"
        );
    }
}

/// Mirroring the ball's velocity VECTOR must equal "yaw + 32768 with pitch
/// (and speed) unchanged" in the archetype's rotator+speed encoding.
#[test]
fn mirrored_ball_velocity_flips_yaw_and_preserves_pitch_and_speed() {
    let ball = sample_ball();
    let (pitch, yaw, _, speed) = velocity_rotator_and_speed(ball.linear_velocity);

    let mut mirrored = ball;
    mirror_ball(&mut mirrored);
    let (mirrored_pitch, mirrored_yaw, _, mirrored_speed) =
        velocity_rotator_and_speed(mirrored.linear_velocity);

    assert_eq!(mirrored_pitch, pitch, "pitch must be preserved");
    assert!(
        (mirrored_speed - speed).abs() < 1e-3,
        "speed must be preserved ({speed} vs {mirrored_speed})"
    );
    // velocity_rotator_and_speed emits yaw in -32768..=32767 (atan2), so
    // compare the two encodings as normalized u16 angles.
    let expected = mirror_yaw(yaw);
    let actual = i64::from(mirrored_yaw).rem_euclid(65536) as i32;
    assert!(
        (actual - expected).abs() <= 1 || (actual - expected).abs() >= 65535,
        "yaw {yaw} should mirror to {expected}, got {actual}"
    );
}

/// A full-round assertion: every car and the ball flip together, and
/// mirroring twice restores the original scenario (up to yaw's u16
/// normalization).
#[test]
fn full_round_mirror_flips_everything_and_double_mirror_restores() {
    let original_ball = sample_ball();
    let original_cars = [sample_car(1), sample_car(0)];

    let mut ball = original_ball;
    let mut cars = original_cars;
    mirror_shot(&mut ball, &mut cars);

    // Everything flipped: no car or ball component escaped the mirror.
    assert_eq!(ball.location.x, -original_ball.location.x);
    assert_eq!(ball.location.y, -original_ball.location.y);
    for (mirrored, original) in cars.iter().zip(&original_cars) {
        assert_eq!(mirrored.location.x, -original.location.x);
        assert_eq!(mirrored.location.y, -original.location.y);
        assert_eq!(mirrored.location.z, original.location.z);
        assert_eq!(mirrored.rotation.yaw, mirror_yaw(original.rotation.yaw));
        assert_eq!(mirrored.rotation.pitch, original.rotation.pitch);
        assert_eq!(mirrored.rotation.roll, original.rotation.roll);
        // Team/primary/boost metadata is untouched by the mirror.
        assert_eq!(mirrored.team, original.team);
        assert_eq!(mirrored.is_primary, original.is_primary);
        assert_eq!(mirrored.boost_amount, original.boost_amount);
    }

    mirror_shot(&mut ball, &mut cars);
    assert_eq!(ball.location, original_ball.location);
    assert_eq!(ball.linear_velocity, original_ball.linear_velocity);
    for (restored, original) in cars.iter().zip(&original_cars) {
        assert_eq!(restored.location, original.location);
        assert_eq!(
            i64::from(restored.rotation.yaw).rem_euclid(65536),
            i64::from(original.rotation.yaw).rem_euclid(65536)
        );
    }
}

/// The decision rule from the corpus derivation: the training player is
/// blue-oriented in both modes, so only orange (team 1) captures mirror.
#[test]
fn should_mirror_only_for_orange_captures_in_both_modes() {
    assert!(!should_mirror(CaptureMode::Shot, 0));
    assert!(should_mirror(CaptureMode::Shot, 1));
    assert!(!should_mirror(CaptureMode::Save, 0));
    assert!(should_mirror(CaptureMode::Save, 1));
}

#[test]
fn capture_mode_decodes_abi_bytes() {
    assert_eq!(CaptureMode::from_abi(0), CaptureMode::Shot);
    assert_eq!(CaptureMode::from_abi(1), CaptureMode::Save);
    // Unknown bytes fall back to the pre-existing shot behavior.
    assert_eq!(CaptureMode::from_abi(7), CaptureMode::Shot);
}
