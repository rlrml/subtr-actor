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
fn touch_candidate_rank_prefers_recent_closest_approach() {
    let ball = sample_rigid_body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let near_but_static = sample_rigid_body(120.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let slightly_farther_but_recently_closer = sample_rigid_body(180.0, 0.0, 0.0, 1500.0, 0.0, 0.0);

    let near_rank = touch_candidate_rank(&ball, &near_but_static).unwrap();
    let recent_rank = touch_candidate_rank(&ball, &slightly_farther_but_recently_closer).unwrap();

    assert!(
        recent_rank < near_rank,
        "expected backtracked closest approach to outrank pure current distance: {recent_rank:?} !< {near_rank:?}"
    );
}

#[test]
fn touch_candidate_rank_penalizes_unreachable_far_candidates() {
    let ball = sample_rigid_body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let close_candidate = sample_rigid_body(200.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let far_candidate = sample_rigid_body(1200.0, 0.0, 0.0, 1000.0, 0.0, 0.0);

    let close_rank = touch_candidate_rank(&ball, &close_candidate).unwrap();
    let far_rank = touch_candidate_rank(&ball, &far_candidate).unwrap();

    assert!(
        close_rank < far_rank,
        "expected a far candidate outside the short contact window to rank worse: {close_rank:?} !< {far_rank:?}"
    );
}
