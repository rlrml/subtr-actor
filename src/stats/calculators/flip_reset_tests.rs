use super::*;
use boxcars::{Quaternion, Vector3f};

fn sample_rigid_body(
    x: f32,
    y: f32,
    z: f32,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: Vector3f { x, y, z },
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(Vector3f {
            x: velocity_x,
            y: velocity_y,
            z: velocity_z,
        }),
        angular_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    }
}

#[test]
fn test_flip_reset_candidate_detects_airborne_underside_touch() {
    let ball = sample_rigid_body(0.0, 0.0, 6.0, 0.0, 0.0, 0.0);
    let player = sample_rigid_body(0.0, 0.0, 8.5, 0.0, 0.0, 0.0);

    let heuristic = flip_reset_candidate(&ball, &player, 1.2)
        .expect("expected underside aerial contact to qualify as a flip-reset candidate");

    assert!(heuristic.confidence > 0.5);
    assert!(heuristic.local_ball_position.z < 0.0);
}

#[test]
fn test_flip_reset_candidate_rejects_front_bumper_like_touch() {
    let ball = sample_rigid_body(7.0, 0.0, 8.5, 0.0, 0.0, 0.0);
    let player = sample_rigid_body(0.0, 0.0, 8.5, 0.0, 0.0, 0.0);

    assert!(
        flip_reset_candidate(&ball, &player, 1.2).is_none(),
        "expected front-facing touch geometry to be rejected"
    );
}

#[test]
fn test_flip_reset_candidate_rejects_low_ground_touch() {
    let ball = sample_rigid_body(0.0, 0.0, 1.2, 0.0, 0.0, 0.0);
    let player = sample_rigid_body(0.0, 0.0, 0.2, 0.0, 0.0, 0.0);

    assert!(
        flip_reset_candidate(&ball, &player, 1.2).is_none(),
        "expected grounded touch geometry to be rejected"
    );
}
