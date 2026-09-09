use super::*;

fn quat_from_axis_angle(axis: glam::Vec3, degrees: f32) -> boxcars::Quaternion {
    let q = glam::Quat::from_axis_angle(axis.normalize(), degrees.to_radians());
    boxcars::Quaternion {
        x: q.x,
        y: q.y,
        z: q.z,
        w: q.w,
    }
}

fn rigid_body(
    position: glam::Vec3,
    velocity: glam::Vec3,
    rotation: boxcars::Quaternion,
) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation,
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(
    position: glam::Vec3,
    velocity: glam::Vec3,
    rotation: boxcars::Quaternion,
    dodge_active: bool,
) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity, rotation)),
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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.05,
        seconds_remaining: None,
    }
}

fn live() -> LivePlayState {
    LivePlayState {
        is_live_play: true,
        ..Default::default()
    }
}

fn step(
    calculator: &mut SpeedFlipCalculator,
    frame_number: usize,
    time: f32,
    sample: PlayerSample,
) {
    calculator
        .update_parts(
            &frame(frame_number, time),
            &GameplayState::default(),
            &BallFrameState::default(),
            &PlayerFrameState {
                players: vec![sample],
            },
            &live(),
        )
        .unwrap();
}

const GROUND_Z: f32 = 17.0;
const FAST: f32 = 2000.0;

/// Drive a player forward on the ground, jump-dodge into the air, play a
/// maneuver described by `forward` / `up` orientations while airborne, then land
/// back on the ground (which is what finalizes the candidate). Returns emitted
/// events.
fn run_maneuver(
    orientations: &[(glam::Vec3, glam::Vec3)],
    velocity: glam::Vec3,
) -> Vec<SpeedFlipEvent> {
    run_maneuver_with_torque(orientations, velocity, None)
}

fn run_maneuver_with_torque(
    orientations: &[(glam::Vec3, glam::Vec3)],
    velocity: glam::Vec3,
    dodge_torque: Option<glam::Vec3>,
) -> Vec<SpeedFlipEvent> {
    let mut calculator = SpeedFlipCalculator::new();
    let identity = quat_from_axis_angle(glam::Vec3::Z, 0.0);
    let ground = glam::Vec3::new(0.0, 0.0, GROUND_Z);
    let air = glam::Vec3::new(0.0, 0.0, 80.0);

    // Two grounded, non-dodging warm-up frames to record ground contact.
    step(
        &mut calculator,
        0,
        0.0,
        player(ground, velocity, identity, false),
    );
    step(
        &mut calculator,
        1,
        0.05,
        player(ground, velocity, identity, false),
    );

    // Dodge rising edge, airborne out of the jump.
    let mut dodge_sample = player(air, velocity, identity, true);
    dodge_sample.dodge_torque = dodge_torque;
    step(&mut calculator, 2, 0.10, dodge_sample);

    // The maneuver: rotate the body frame by frame while airborne. We
    // approximate orientation by building a rotation whose forward/up match the
    // requested vectors.
    let mut t = 0.15;
    let mut n = 3;
    for (forward, up) in orientations {
        let rotation = rotation_from_forward_up(*forward, *up);
        step(&mut calculator, n, t, player(air, velocity, rotation, true));
        t += 0.05;
        n += 1;
    }

    // Land back on the ground: touching down ends the maneuver and finalizes.
    step(
        &mut calculator,
        n,
        t,
        player(
            ground,
            velocity,
            quat_from_axis_angle(glam::Vec3::Z, 0.0),
            false,
        ),
    );
    calculator.finalize_parts(&frame(n + 1, t + 0.1));
    calculator.events().to_vec()
}

fn rotation_from_forward_up(forward: glam::Vec3, up: glam::Vec3) -> boxcars::Quaternion {
    let f = forward.normalize();
    let u = up.normalize();
    let right = u.cross(f).normalize();
    let up = f.cross(right).normalize();
    let mat = glam::Mat3::from_cols(f, right, up);
    let q = glam::Quat::from_mat3(&mat);
    boxcars::Quaternion {
        x: q.x,
        y: q.y,
        z: q.z,
        w: q.w,
    }
}

#[test]
fn accepts_forward_diagonal_roll_to_recover() {
    // Nose stays near +X (aligned with travel) while the up vector rolls a full
    // revolution: a clean speed flip.
    let forward = glam::Vec3::X;
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let roll = (i as f32) * 36.0_f32.to_radians();
            // small forward dip, big roll about the nose axis
            let up = glam::Vec3::new(-0.2, 0.0, 1.0).normalize();
            let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * up;
            (forward, up)
        })
        .collect();

    let events = run_maneuver(&orientations, glam::Vec3::new(FAST, 0.0, 0.0));
    assert_eq!(events.len(), 1, "expected one speed flip, got {events:#?}");
    let event = &events[0];
    assert!(event.max_forward_deviation_degrees <= SPEED_FLIP_MAX_FORWARD_DEVIATION_DEGREES);
    assert!(event.roll_sweep_degrees >= SPEED_FLIP_MIN_ROLL_SWEEP_DEGREES);
    assert!(event.min_travel_alignment >= SPEED_FLIP_MIN_TRAVEL_ALIGNMENT);
}

#[test]
fn replicated_dodge_torque_requires_a_forward_diagonal_input() {
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let roll = (i as f32) * 36.0_f32.to_radians();
            let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * glam::Vec3::Z;
            (glam::Vec3::X, up)
        })
        .collect();

    let diagonal = run_maneuver_with_torque(
        &orientations,
        glam::Vec3::new(FAST, 0.0, 0.0),
        Some(glam::Vec3::new(1.84, 1.84, 0.0)),
    );
    assert_eq!(diagonal.len(), 1);
    assert_eq!(diagonal[0].dodge_side_component, 1.84);

    for torque in [
        glam::Vec3::new(0.0, 2.6, 0.0),
        glam::Vec3::new(2.6, 0.0, 0.0),
        glam::Vec3::new(1.84, -1.84, 0.0),
    ] {
        let events =
            run_maneuver_with_torque(&orientations, glam::Vec3::new(FAST, 0.0, 0.0), Some(torque));
        assert!(
            events.is_empty(),
            "non-forward-diagonal torque should be rejected: {torque:?}"
        );
    }
}

#[test]
fn near_complete_roll_requires_alignment_or_a_strong_diagonal_input() {
    let maneuver = |heading_degrees: f32, roll_degrees: f32| {
        let forward = glam::Quat::from_axis_angle(glam::Vec3::Z, heading_degrees.to_radians())
            * glam::Vec3::X;
        (0..10)
            .map(|i| {
                let roll = roll_degrees * (i as f32 / 9.0);
                let up = glam::Quat::from_axis_angle(forward, roll.to_radians()) * glam::Vec3::Z;
                (forward, up)
            })
            .collect::<Vec<_>>()
    };

    let moderately_aligned = maneuver(30.0, 175.0);
    let strong_diagonal = run_maneuver_with_torque(
        &moderately_aligned,
        glam::Vec3::new(FAST, 0.0, 0.0),
        Some(glam::Vec3::new(1.84, 1.84, 0.0)),
    );
    assert_eq!(strong_diagonal.len(), 1);

    let weak_diagonal = run_maneuver_with_torque(
        &moderately_aligned,
        glam::Vec3::new(FAST, 0.0, 0.0),
        Some(glam::Vec3::new(2.3, 1.1, 0.0)),
    );
    assert!(weak_diagonal.is_empty());

    let poorly_aligned = run_maneuver_with_torque(
        &maneuver(45.0, 168.0),
        glam::Vec3::new(FAST, 0.0, 0.0),
        Some(glam::Vec3::new(1.84, 1.84, 0.0)),
    );
    assert!(poorly_aligned.is_empty());
}

#[test]
fn opening_kickoff_window_stays_open_until_the_first_ball_hit() {
    let mut calculator = SpeedFlipCalculator::default();

    assert!(calculator.update_kickoff_window(&GameplayState {
        kickoff_countdown_time: Some(3),
        ..Default::default()
    }));
    assert!(calculator.update_kickoff_window(&GameplayState::default()));
    assert!(!calculator.update_kickoff_window(&GameplayState {
        ball_has_been_hit: Some(true),
        ..Default::default()
    }));
}

#[test]
fn rejects_front_flip_that_goes_end_over_end() {
    // The nose pitches all the way over (end-over-end): forward sweeps ~180.
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let pitch = (i as f32) * 18.0_f32.to_radians();
            let rot = glam::Quat::from_axis_angle(glam::Vec3::Y, pitch);
            (rot * glam::Vec3::X, rot * glam::Vec3::Z)
        })
        .collect();

    let events = run_maneuver(&orientations, glam::Vec3::new(FAST, 0.0, 0.0));
    assert!(
        events.is_empty(),
        "front flip should be rejected, got {events:#?}"
    );
}

#[test]
fn rejects_flat_wavedash_without_roll() {
    // Nose stays put and the car barely rolls: a wavedash, not a speed flip.
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let roll = (i as f32) * 3.0_f32.to_radians();
            let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * glam::Vec3::Z;
            (glam::Vec3::X, up)
        })
        .collect();

    let events = run_maneuver(&orientations, glam::Vec3::new(FAST, 0.0, 0.0));
    assert!(
        events.is_empty(),
        "wavedash should be rejected, got {events:#?}"
    );
}

#[test]
fn rejects_when_nose_drifts_off_travel_direction() {
    // The car rolls plenty, but the nose veers ~70 degrees off the travel
    // direction: not a speed flip (criterion c).
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let yaw = (i as f32) * 8.0_f32.to_radians();
            let roll = (i as f32) * 20.0_f32.to_radians();
            let forward = glam::Quat::from_axis_angle(glam::Vec3::Z, yaw) * glam::Vec3::X;
            let up = glam::Quat::from_axis_angle(forward, roll) * glam::Vec3::Z;
            (forward, up)
        })
        .collect();

    // Travel stays along +X while the nose yaws away.
    let events = run_maneuver(&orientations, glam::Vec3::new(FAST, 0.0, 0.0));
    assert!(
        events.is_empty(),
        "nose drifting off travel should be rejected, got {events:#?}"
    );
}

#[test]
fn rejects_slow_dodge_that_does_not_reach_speed() {
    let forward = glam::Vec3::X;
    let orientations: Vec<(glam::Vec3, glam::Vec3)> = (0..10)
        .map(|i| {
            let roll = (i as f32) * 36.0_f32.to_radians();
            let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * glam::Vec3::Z;
            (forward, up)
        })
        .collect();

    // Below SPEED_FLIP_MIN_MAX_SPEED throughout.
    let events = run_maneuver(&orientations, glam::Vec3::new(900.0, 0.0, 0.0));
    assert!(
        events.is_empty(),
        "slow dodge should be rejected, got {events:#?}"
    );
}

#[test]
fn rejects_ground_dodge_that_stays_airborne_too_long() {
    // A clean roll-to-recover orientation, but the car never comes back down
    // within the airborne budget: an aerial, not a speed flip.
    let mut calculator = SpeedFlipCalculator::new();
    let identity = quat_from_axis_angle(glam::Vec3::Z, 0.0);
    let ground = glam::Vec3::new(0.0, 0.0, GROUND_Z);
    let air = glam::Vec3::new(0.0, 0.0, 300.0);
    let velocity = glam::Vec3::new(FAST, 0.0, 0.0);

    step(
        &mut calculator,
        0,
        0.0,
        player(ground, velocity, identity, false),
    );
    step(
        &mut calculator,
        1,
        0.05,
        player(ground, velocity, identity, false),
    );
    step(
        &mut calculator,
        2,
        0.10,
        player(air, velocity, identity, true),
    );

    // Stay airborne, rolling, well past SPEED_FLIP_MAX_AIRBORNE_SECONDS.
    let mut t = 0.15;
    let mut n = 3;
    while t - 0.10 <= SPEED_FLIP_MAX_AIRBORNE_SECONDS + 0.3 {
        let roll = (n as f32) * 18.0_f32.to_radians();
        let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * glam::Vec3::Z;
        let rotation = rotation_from_forward_up(glam::Vec3::X, up);
        step(&mut calculator, n, t, player(air, velocity, rotation, true));
        t += 0.05;
        n += 1;
    }
    calculator.finalize_parts(&frame(n + 1, t + 0.1));

    assert!(
        calculator.events().is_empty(),
        "a dodge that never lands in time should be rejected, got {:#?}",
        calculator.events()
    );
}

#[test]
fn rejects_aerial_dodge_not_started_from_ground() {
    let mut calculator = SpeedFlipCalculator::new();
    let identity = quat_from_axis_angle(glam::Vec3::Z, 0.0);
    let airborne = glam::Vec3::new(0.0, 0.0, 500.0);
    let velocity = glam::Vec3::new(FAST, 0.0, 0.0);

    // Never grounded; dodge fires in the air.
    step(
        &mut calculator,
        0,
        0.0,
        player(airborne, velocity, identity, false),
    );
    step(
        &mut calculator,
        1,
        0.05,
        player(airborne, velocity, identity, true),
    );
    for i in 0..10 {
        let roll = (i as f32) * 36.0_f32.to_radians();
        let up = glam::Quat::from_axis_angle(glam::Vec3::X, roll) * glam::Vec3::Z;
        let rotation = rotation_from_forward_up(glam::Vec3::X, up);
        step(
            &mut calculator,
            2 + i,
            0.10 + (i as f32) * 0.05,
            player(airborne, velocity, rotation, true),
        );
    }
    calculator.finalize_parts(&frame(99, 5.0));

    assert!(
        calculator.events().is_empty(),
        "aerial dodge should not be a speed flip, got {:#?}",
        calculator.events()
    );
}
