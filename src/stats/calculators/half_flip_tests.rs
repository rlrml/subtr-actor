use super::*;

fn quaternion_from_yaw(yaw: f32) -> boxcars::Quaternion {
    let rotation = glam::Quat::from_rotation_z(yaw);
    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

fn rigid_body(
    position: glam::Vec3,
    velocity: glam::Vec3,
    yaw: f32,
    forward_pitch: f32,
) -> boxcars::RigidBody {
    let rotation = glam::Quat::from_rotation_z(yaw) * glam::Quat::from_rotation_y(forward_pitch);
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: rotation.x,
            y: rotation.y,
            z: rotation.z,
            w: rotation.w,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(
    id: u64,
    velocity: glam::Vec3,
    yaw: f32,
    forward_pitch: f32,
    dodge_active: bool,
) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(
            glam::Vec3::new(0.0, 0.0, 20.0),
            velocity,
            yaw,
            forward_pitch,
        )),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.05,
        seconds_remaining: None,
    }
}

fn players(player: PlayerSample) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player],
    }
}

#[test]
fn counts_backward_dodge_that_reorients_forward() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, glam::Vec3::new(-600.0, 0.0, 0.0), 0.0, 0.0, true)),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player(
                1,
                glam::Vec3::new(-900.0, 0.0, 0.0),
                0.0,
                std::f32::consts::FRAC_PI_2,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player(
                1,
                glam::Vec3::new(-1300.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert!(stats.last_quality.unwrap() >= HALF_FLIP_MIN_CONFIDENCE);
    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].frame, 15);
    assert!(calculator.events()[0].start_backward_alignment >= 0.99);
    assert!(calculator.events()[0].best_reorientation_alignment >= 0.99);
}

#[test]
fn rejects_forward_dodge_start() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, glam::Vec3::new(900.0, 0.0, 0.0), 0.0, 0.0, true)),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player(1, glam::Vec3::new(1300.0, 0.0, 0.0), 0.0, 0.0, true)),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_backward_dodge_without_reorientation() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, glam::Vec3::new(-600.0, 0.0, 0.0), 0.0, 0.0, true)),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(PlayerSample {
                rigid_body: Some(boxcars::RigidBody {
                    rotation: quaternion_from_yaw(0.0),
                    ..rigid_body(
                        glam::Vec3::new(0.0, 0.0, 20.0),
                        glam::Vec3::new(-900.0, 0.0, 0.0),
                        0.0,
                        0.0,
                    )
                }),
                ..player(1, glam::Vec3::new(-900.0, 0.0, 0.0), 0.0, 0.0, true)
            }),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}
