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
    player_at_z(id, 20.0, velocity, yaw, forward_pitch, dodge_active)
}

fn player_at_z(
    id: u64,
    z: f32,
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
            glam::Vec3::new(0.0, 0.0, z),
            velocity,
            yaw,
            forward_pitch,
        )),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn player_with_spin(
    id: u64,
    z: f32,
    velocity: glam::Vec3,
    yaw: f32,
    forward_pitch: f32,
    angular_velocity: glam::Vec3,
    dodge_active: bool,
) -> PlayerSample {
    let mut sample = player_at_z(id, z, velocity, yaw, forward_pitch, dodge_active);
    if let Some(rigid_body) = sample.rigid_body.as_mut() {
        rigid_body.angular_velocity = Some(glam_to_vec(&angular_velocity));
    }
    sample
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
fn counts_low_jump_half_flip_start() {
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player_at_z(
                1,
                82.0,
                glam::Vec3::new(-520.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_at_z(
                1,
                70.0,
                glam::Vec3::new(-520.0, 0.0, 0.0),
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
            &players(player_at_z(
                1,
                55.0,
                glam::Vec3::new(-520.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn counts_slow_orientation_clear_half_flip() {
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player_at_z(
                1,
                74.0,
                glam::Vec3::new(-120.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_at_z(
                1,
                62.0,
                glam::Vec3::new(-120.0, 0.0, 0.0),
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
            &players(player_at_z(
                1,
                42.0,
                glam::Vec3::new(-120.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn counts_forward_travel_when_facing_reverses() {
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player_at_z(
                1,
                74.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_at_z(
                1,
                62.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
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
            &players(player_at_z(
                1,
                42.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].start_backward_alignment < 0.0);
}

#[test]
fn counts_backward_dodge_without_start_height_filter() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(
                1,
                glam::Vec3::new(-450.0, 0.0, 0.0),
                0.0,
                0.0,
                false,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 1.05),
            &players(player_at_z(
                1,
                PLAYER_GROUND_Z_THRESHOLD + 250.0,
                glam::Vec3::new(-450.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(9, 1.40),
            &players(player_at_z(
                1,
                PLAYER_GROUND_Z_THRESHOLD + 320.0,
                glam::Vec3::new(-450.0, 0.0, 0.0),
                0.0,
                std::f32::consts::FRAC_PI_2,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(16, 1.75),
            &players(player_at_z(
                1,
                PLAYER_GROUND_Z_THRESHOLD + 280.0,
                glam::Vec3::new(-450.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].frame, 16);
    assert!(calculator.events()[0].confidence >= HALF_FLIP_MIN_CONFIDENCE);
}

#[test]
fn rejects_dodge_without_facing_reversal() {
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
fn rejects_flip_that_has_not_cancelled_back_to_horizontal_facing() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player_at_z(
                1,
                74.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player_at_z(
                1,
                62.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                std::f32::consts::PI,
                std::f32::consts::FRAC_PI_2,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_end_over_end_flip_that_rotates_away_after_reaching_opposite() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player_at_z(
                1,
                74.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(5, 1.20),
            &players(player_at_z(
                1,
                72.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                std::f32::consts::FRAC_PI_2,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_at_z(
                1,
                66.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(11, 1.50),
            &players(player_at_z(
                1,
                58.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player_at_z(
                1,
                42.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_air_hit_tumble_dodge_where_spin_precedes_the_dodge() {
    // Failed wave dash after getting bumped in the air: the car is already
    // tumbling hard (and traveling forward into the spin) *before* the dodge
    // byte fires, then somersaults through vertical and lands facing the other
    // way. The facing reversal was not produced by the dodge, so it must not be
    // counted as a half-flip.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    // Pre-dodge frame: spinning fast, nose flat, moving forward (facing +X).
    calculator
        .update(
            &frame(1, 1.00),
            &players(player_with_spin(
                1,
                90.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                0.0,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                false,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    // Dodge byte fires while still traveling forward into the spin.
    calculator
        .update(
            &frame(2, 1.05),
            &players(player_with_spin(
                1,
                95.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                0.0,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_with_spin(
                1,
                95.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                0.0,
                std::f32::consts::FRAC_PI_2,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player_with_spin(
                1,
                80.0,
                glam::Vec3::new(600.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn counts_deliberate_backward_half_flip_despite_elevated_pre_dodge_spin() {
    // A real backward half-flip entry (velocity opposes facing) must still count
    // even with elevated pre-dodge spin: the air-hit guard only rejects
    // forward-entry tumbles, not deliberate reversals.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfFlipCalculator::new();

    // Pre-dodge frame: spinning, but driving backward (facing +X, moving -X).
    calculator
        .update(
            &frame(1, 1.00),
            &players(player_with_spin(
                1,
                74.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                false,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 1.05),
            &players(player_with_spin(
                1,
                72.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(8, 1.35),
            &players(player_with_spin(
                1,
                62.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                0.0,
                std::f32::consts::FRAC_PI_2,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(15, 1.70),
            &players(player_with_spin(
                1,
                42.0,
                glam::Vec3::new(-600.0, 0.0, 0.0),
                std::f32::consts::PI,
                0.0,
                glam::Vec3::new(0.0, 0.0, 500.0),
                true,
            )),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].start_backward_alignment >= 0.0);
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
