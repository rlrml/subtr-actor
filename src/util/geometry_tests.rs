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

fn sample_rotated_rigid_body(x: f32, y: f32, z: f32, rotation: glam::Quat) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: Vector3f { x, y, z },
        rotation: glam_to_quat(&rotation),
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
fn apply_velocities_to_rigid_body_applies_and_normalizes_angular_velocity() {
    let mut body = sample_rigid_body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    body.angular_velocity = Some(Vector3f {
        x: 0.0,
        y: 0.0,
        z: std::f32::consts::FRAC_PI_2,
    });

    let updated = apply_velocities_to_rigid_body(&body, 1.0);
    let rotation = quat_to_glam(&updated.rotation);
    let rotated_forward = rotation * glam::Vec3::X;

    assert!((rotation.length() - 1.0).abs() <= 0.001);
    assert!(rotated_forward.x.abs() <= 0.001);
    assert!((rotated_forward.y - 1.0).abs() <= 0.001);
}

#[test]
fn apply_velocities_to_rigid_body_uses_identity_for_degenerate_rotation() {
    let mut body = sample_rigid_body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    body.rotation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    body.angular_velocity = Some(Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });

    let updated = apply_velocities_to_rigid_body(&body, 1.0);

    assert_eq!(updated.rotation, glam_to_quat(&glam::Quat::IDENTITY));
}

#[test]
fn quat_to_glam_normalizes_scaled_rotation() {
    let rotation = glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let scaled_rotation = Quaternion {
        x: rotation.x * 2.0,
        y: rotation.y * 2.0,
        z: rotation.z * 2.0,
        w: rotation.w * 2.0,
    };

    let normalized = quat_to_glam(&scaled_rotation);
    let rotated_forward = normalized * glam::Vec3::X;

    assert!((normalized.length() - 1.0).abs() <= 0.001);
    assert!(rotated_forward.x.abs() <= 0.001);
    assert!((rotated_forward.y - 1.0).abs() <= 0.001);
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
fn touch_candidate_rank_applies_car_angular_velocity_to_hitbox_orientation() {
    let hitbox = default_car_hitbox();
    let ball_position = glam::Vec3::new(
        hitbox.length / 2.0 + BALL_COLLISION_RADIUS,
        0.0,
        hitbox.elevation,
    );
    let ball = sample_rigid_body(
        ball_position.x,
        ball_position.y,
        ball_position.z,
        0.0,
        0.0,
        0.0,
    );
    let current_sideways_player = sample_rotated_rigid_body(
        0.0,
        0.0,
        0.0,
        glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    let mut rotating_player = current_sideways_player;
    rotating_player.angular_velocity = Some(Vector3f {
        x: 0.0,
        y: 0.0,
        z: std::f32::consts::FRAC_PI_2 / 0.1,
    });

    let static_rank =
        touch_candidate_contact_gap_rank_with_hitbox(&ball, &current_sideways_player, hitbox)
            .unwrap();
    let rotating_rank =
        touch_candidate_contact_gap_rank_with_hitbox(&ball, &rotating_player, hitbox).unwrap();

    assert!(
        rotating_rank.0 < static_rank.0,
        "expected sampled angular velocity to find the earlier hitbox orientation: {rotating_rank:?} !< {static_rank:?}"
    );
}

#[test]
fn ball_trajectory_deviation_with_gravity_is_small_for_expected_gravity_motion() {
    let start = sample_rigid_body(0.0, 0.0, 1000.0, 100.0, 0.0, 0.0);
    let actual = sample_rigid_body(10.0, 0.0, 996.75, 100.0, 0.0, -65.0);

    let deviation =
        ball_trajectory_deviation_with_gravity(&start, 1.0, &actual, 1.1, -650.0).unwrap();

    assert!(deviation.position_deviation < 0.001);
    assert!(deviation.velocity_deviation < 0.001);
}

#[test]
fn ball_trajectory_deviation_with_gravity_detects_impulse_like_motion() {
    let start = sample_rigid_body(0.0, 0.0, 1000.0, 100.0, 0.0, 0.0);
    let actual = sample_rigid_body(80.0, 0.0, 996.75, 900.0, 0.0, -65.0);

    let deviation =
        ball_trajectory_deviation_with_gravity(&start, 1.0, &actual, 1.1, -650.0).unwrap();

    assert!(deviation.position_deviation >= 70.0);
    assert!(deviation.velocity_deviation > 700.0);
}

#[test]
fn touch_candidate_scoring_requires_ball_deviation_for_contact_gaps() {
    let scoring = TouchCandidateScoring::DEFAULT;

    assert!(!scoring.accepts_contact_gap(0.0, 0.0, 0.0));
    assert!(scoring.accepts_contact_gap(0.0, 0.0, 50.0));
    assert!(scoring.accepts_contact_gap(0.0, 25.0, 0.0));
    assert!(!scoring.accepts_contact_gap(10.0, 499.0, 999.0));
    assert!(scoring.accepts_contact_gap(10.0, 0.0, 1000.0));
    assert!(scoring.accepts_contact_gap(10.0, 500.0, 0.0));
    assert!(
        scoring.score_contact_gap(10.0, false) > scoring.score_contact_gap(5.0, false),
        "relaxed candidates should rank behind strict candidates"
    );
}

#[test]
fn car_hitbox_distance_uses_car_orientation() {
    let ball_position = glam::Vec3::new(0.0, 70.0, 17.0);
    let forward_car = sample_rotated_rigid_body(0.0, 0.0, 0.0, glam::Quat::from_rotation_z(0.0));
    let sideways_car = sample_rotated_rigid_body(
        0.0,
        0.0,
        0.0,
        glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );

    let forward_distance =
        car_hitbox_distance(ball_position, &forward_car, default_car_hitbox()).unwrap();
    let sideways_distance =
        car_hitbox_distance(ball_position, &sideways_car, default_car_hitbox()).unwrap();

    assert!(
        sideways_distance < forward_distance,
        "expected the rotated car's longer axis to make the same ball position closer to its hitbox: {sideways_distance:?} !< {forward_distance:?}"
    );
}

#[test]
fn car_hitbox_distance_normalizes_scaled_car_orientation() {
    let hitbox = default_car_hitbox();
    let ball_position = glam::Vec3::new(0.0, 70.0, 17.0);
    let rotation = glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let unit_car = sample_rotated_rigid_body(0.0, 0.0, 0.0, rotation);
    let mut scaled_car = unit_car;
    scaled_car.rotation = Quaternion {
        x: unit_car.rotation.x * 2.0,
        y: unit_car.rotation.y * 2.0,
        z: unit_car.rotation.z * 2.0,
        w: unit_car.rotation.w * 2.0,
    };

    let unit_distance = car_hitbox_distance(ball_position, &unit_car, hitbox).unwrap();
    let scaled_distance = car_hitbox_distance(ball_position, &scaled_car, hitbox).unwrap();

    assert!((unit_distance - scaled_distance).abs() <= 0.001);
}

#[test]
fn car_hitbox_ball_contact_gap_subtracts_ball_radius() {
    let hitbox = default_car_hitbox();
    let car = sample_rotated_rigid_body(0.0, 0.0, 0.0, glam::Quat::IDENTITY);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let front_normal = hitbox_rotation * glam::Vec3::X;
    let touching_ball_center =
        hitbox_center + front_normal * (hitbox.length / 2.0 + BALL_COLLISION_RADIUS);
    let separated_ball_center = touching_ball_center + front_normal * 10.0;

    let touching_gap = car_hitbox_ball_contact_gap(touching_ball_center, &car, hitbox).unwrap();
    let separated_gap = car_hitbox_ball_contact_gap(separated_ball_center, &car, hitbox).unwrap();

    assert!(touching_gap <= 0.001);
    assert!((separated_gap - 10.0).abs() <= 0.001);
}

#[test]
fn car_hitbox_distance_applies_hitbox_offset_elevation_and_slope() {
    let hitbox = default_car_hitbox();
    let car = sample_rotated_rigid_body(0.0, 0.0, 0.0, glam::Quat::IDENTITY);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let top_center =
        hitbox_center + hitbox_rotation * glam::Vec3::new(0.0, 0.0, hitbox.height / 2.0);

    let distance = car_hitbox_distance(top_center, &car, hitbox).unwrap();

    assert!(distance <= 0.001);
}

#[test]
fn car_hitbox_distance_composes_car_rotation_with_hitbox_transform() {
    let hitbox = default_car_hitbox();
    let car_position = glam::Vec3::new(1000.0, -500.0, 300.0);
    let car_rotation = glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let car =
        sample_rotated_rigid_body(car_position.x, car_position.y, car_position.z, car_rotation);
    let hitbox_center = glam::Vec3::new(hitbox.offset, 0.0, hitbox.elevation);
    let hitbox_rotation = glam::Quat::from_rotation_y(hitbox.angle.to_radians());
    let local_front_top_corner =
        glam::Vec3::new(hitbox.length / 2.0, hitbox.width / 2.0, hitbox.height / 2.0);
    let world_corner =
        car_position + car_rotation * (hitbox_center + hitbox_rotation * local_front_top_corner);

    let distance = car_hitbox_distance(world_corner, &car, hitbox).unwrap();

    assert!(distance <= 0.001);
}

#[test]
fn car_hitbox_floor_contact_uses_hitbox_bottom() {
    let hitbox = default_car_hitbox();
    let grounded_car = sample_rotated_rigid_body(0.0, 0.0, 0.0, glam::Quat::IDENTITY);
    let airborne_car = sample_rotated_rigid_body(0.0, 0.0, 100.0, glam::Quat::IDENTITY);

    assert!(car_hitbox_min_world_z(&grounded_car, hitbox).unwrap() <= 5.0);
    assert!(car_hitbox_touches_floor(&grounded_car, hitbox));
    assert!(!car_hitbox_touches_floor(&airborne_car, hitbox));
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
fn car_hitbox_for_body_id_or_name_prefers_id_and_falls_back_to_name() {
    assert_eq!(
        hitbox_family_for_body_id_or_name(Some(23), Some("Dominus GT")),
        Some(CarHitboxFamily::Octane)
    );
    assert_eq!(
        hitbox_family_for_body_id_or_name(Some(999_999), Some("Dominus GT")),
        Some(CarHitboxFamily::Dominus)
    );
    assert_eq!(hitbox_family_for_body_id_or_name(Some(999_999), None), None);
    assert_eq!(
        hitbox_family_for_body_id_or_name(None, Some("unknown body")),
        None
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
