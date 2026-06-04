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
fn interpolates_rigid_body_location() {
    let start_body = boxcars::RigidBody {
        sleeping: false,
        location: Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
        angular_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    };
    let end_body = boxcars::RigidBody {
        sleeping: true,
        location: Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 1.0,
            w: 0.0,
        },
        linear_velocity: Some(Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
        angular_velocity: Some(Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
    };

    let interpolated_body = get_interpolated_rigid_body(&start_body, 0.0, &end_body, 1.0, 0.5)
        .expect("interpolation should succeed");

    assert_eq!(interpolated_body.location.x, 0.5);
    assert_eq!(interpolated_body.location.y, 0.5);
    assert_eq!(interpolated_body.location.z, 0.5);
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

#[test]
fn car_hitbox_for_body_name_maps_known_cars_to_hitbox_families() {
    assert_eq!(
        car_hitbox_for_body_name("Fennec").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Octane)
    );
    assert_eq!(
        car_hitbox_for_body_name("Dominus GT").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Dominus)
    );
    assert_eq!(
        car_hitbox_for_body_name("Ford Mustang Shelby GT500").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Dominus)
    );
    assert_eq!(
        car_hitbox_for_body_name("Porsche 918 Spyder").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Breakout)
    );
    assert_eq!(
        car_hitbox_for_body_name("Nissan Skyline GT-R (R32)").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Hybrid)
    );
    assert_eq!(
        car_hitbox_for_body_name("The Mystery Machine").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Merc)
    );
    assert_eq!(
        car_hitbox_for_body_name("'16 Batmobile").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Plank)
    );
    assert_eq!(
        car_hitbox_for_body_name("Aston Martin Valhalla").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Breakout)
    );
    assert_eq!(
        car_hitbox_for_body_name("BMW M2 Racing").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Dominus)
    );
    assert_eq!(
        car_hitbox_for_body_name("Rivian R1S").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Hybrid)
    );
    assert_eq!(
        car_hitbox_for_body_name("Pizza Planet Delivery Truck").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Merc)
    );
    assert_eq!(
        car_hitbox_for_body_name("Psyclops").map(|hitbox| hitbox.family),
        Some(CarHitboxFamily::Octane)
    );
}

#[test]
fn car_hitbox_for_body_id_maps_bakkesmod_carbody_ids() {
    let expected = [
        (21, CarHitboxFamily::Octane),
        (22, CarHitboxFamily::Breakout),
        (1416, CarHitboxFamily::Breakout),
        (23, CarHitboxFamily::Octane),
        (1568, CarHitboxFamily::Octane),
        (24, CarHitboxFamily::Plank),
        (25, CarHitboxFamily::Octane),
        (1300, CarHitboxFamily::Octane),
        (26, CarHitboxFamily::Octane),
        (27, CarHitboxFamily::Octane),
        (28, CarHitboxFamily::Hybrid),
        (1159, CarHitboxFamily::Hybrid),
        (29, CarHitboxFamily::Dominus),
        (30, CarHitboxFamily::Merc),
        (31, CarHitboxFamily::Hybrid),
        (402, CarHitboxFamily::Octane),
        (1295, CarHitboxFamily::Octane),
        (403, CarHitboxFamily::Dominus),
        (1018, CarHitboxFamily::Dominus),
        (404, CarHitboxFamily::Octane),
        (523, CarHitboxFamily::Octane),
        (597, CarHitboxFamily::Dominus),
        (600, CarHitboxFamily::Dominus),
        (607, CarHitboxFamily::Octane),
        (625, CarHitboxFamily::Octane),
        (723, CarHitboxFamily::Octane),
        (803, CarHitboxFamily::Plank),
        (1171, CarHitboxFamily::Dominus),
        (1172, CarHitboxFamily::Octane),
        (1286, CarHitboxFamily::Dominus),
        (1317, CarHitboxFamily::Hybrid),
        (1475, CarHitboxFamily::Octane),
        (1478, CarHitboxFamily::Octane),
        (1533, CarHitboxFamily::Octane),
        (1603, CarHitboxFamily::Plank),
        (1623, CarHitboxFamily::Octane),
        (1624, CarHitboxFamily::Hybrid),
        (1675, CarHitboxFamily::Dominus),
        (1691, CarHitboxFamily::Plank),
        (1856, CarHitboxFamily::Hybrid),
        (1919, CarHitboxFamily::Plank),
        (1932, CarHitboxFamily::Breakout),
    ];

    for (body_id, family) in expected {
        assert_eq!(
            hitbox_family_for_body_id(body_id),
            Some(family),
            "body id {body_id}"
        );
    }
}

#[test]
fn car_hitbox_for_body_id_maps_newer_fixture_body_ids() {
    let expected = [
        (4782, CarHitboxFamily::Octane),
        (8565, CarHitboxFamily::Breakout),
        (8566, CarHitboxFamily::Breakout),
        (9140, CarHitboxFamily::Dominus),
        (9388, CarHitboxFamily::Dominus),
        (10817, CarHitboxFamily::Breakout),
        (11095, CarHitboxFamily::Dominus),
        (11141, CarHitboxFamily::Hybrid),
        (11315, CarHitboxFamily::Dominus),
        (11336, CarHitboxFamily::Dominus),
        (12315, CarHitboxFamily::Breakout),
        (12325, CarHitboxFamily::Dominus),
        (12335, CarHitboxFamily::Merc),
        (12382, CarHitboxFamily::Dominus),
        (12484, CarHitboxFamily::Breakout),
        (12563, CarHitboxFamily::Dominus),
        (12569, CarHitboxFamily::Hybrid),
        (12652, CarHitboxFamily::Hybrid),
        (12669, CarHitboxFamily::Dominus),
    ];

    for (body_id, family) in expected {
        assert_eq!(
            hitbox_family_for_body_id(body_id),
            Some(family),
            "body id {body_id}"
        );
    }
}

#[test]
fn car_hitbox_for_body_id_ignores_placeholder_product_ids() {
    for body_id in [1412, 3138, 3315, 3316, 5364, 5365, 5366, 5367, 5368, 5369] {
        assert_eq!(
            hitbox_family_for_body_id(body_id),
            None,
            "body id {body_id}"
        );
    }
}
