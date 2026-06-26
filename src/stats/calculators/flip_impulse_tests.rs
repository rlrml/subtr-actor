use super::*;

fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(velocity: glam::Vec3, dodge_active: bool, boost_active: bool) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(glam::Vec3::new(0.0, 0.0, 17.0), velocity)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active,
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

fn frame(frame_number: usize, time: f32, dt: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt,
        seconds_remaining: None,
    }
}

#[test]
fn emits_forward_left_impulse_from_velocity_delta() {
    let mut calculator = FlipImpulseCalculator::new();
    let live_play = LivePlayState::active_play();

    calculator
        .update_parts(
            &frame(10, 1.0, 0.0),
            &PlayerFrameState {
                players: vec![player(glam::Vec3::new(1000.0, 0.0, 0.0), true, false)],
            },
            &live_play,
        )
        .unwrap();
    calculator
        .update_parts(
            &frame(12, 1.10, 0.10),
            &PlayerFrameState {
                players: vec![player(glam::Vec3::new(1220.0, -120.0, 0.0), true, false)],
            },
            &live_play,
        )
        .unwrap();
    calculator
        .update_parts(
            &frame(14, 1.20, 0.10),
            &PlayerFrameState {
                players: vec![player(glam::Vec3::new(1220.0, -120.0, 0.0), false, false)],
            },
            &live_play,
        )
        .unwrap();
    // The dodge now resolves at the longer rotation window; force completion.
    calculator.finalize_parts(&frame(16, 1.30, 0.10));

    let event = calculator.events().first().expect("expected event");
    assert_eq!(event.frame, 10);
    // resolved_* still mark the impulse window's end, not the rotation window.
    assert_eq!(event.resolved_frame, 12);
    let dodge_impulse = event
        .dodge_impulse
        .as_ref()
        .expect("expected dodge impulse");
    assert_eq!(dodge_impulse.direction_label, "forward_left");
    assert!(dodge_impulse.local_forward_component > 0.8);
    assert!(dodge_impulse.local_right_component < -0.4);
}

#[test]
fn subtracts_forward_boost_compensation() {
    let candidate = ActiveFlipImpulseCandidate {
        is_team_0: true,
        start_time: 1.0,
        start_frame: 10,
        latest_time: 1.1,
        latest_frame: 12,
        start_position: glam::Vec3::ZERO,
        end_position: glam::Vec3::ZERO,
        start_velocity: glam::Vec3::new(1000.0, 0.0, 0.0),
        end_velocity: glam::Vec3::new(1200.0, 100.0, 0.0),
        local_forward: glam::Vec3::X,
        local_right: glam::Vec3::Y,
        local_up: glam::Vec3::Z,
        boost_compensation: glam::Vec3::new(100.0, 0.0, 0.0),
        sample_count: 2,
        boost_sample_count: 1,
        onset_local_angular_velocity: glam::Vec3::ZERO,
        min_forward_z: 0.0,
        max_forward_deviation_degrees: 0.0,
        max_up_deviation_degrees: 0.0,
        min_up_z: 1.0,
        rotation_sample_count: 0,
        dodge_torque: None,
    };

    let event = FlipImpulseCalculator::candidate_event(&boxcars::RemoteId::Steam(1), candidate);
    let dodge_impulse = event
        .dodge_impulse
        .as_ref()
        .expect("expected dodge impulse");

    assert_eq!(dodge_impulse.raw_velocity_delta, [200.0, 100.0, 0.0]);
    assert_eq!(dodge_impulse.estimated_impulse_delta, [100.0, 100.0, 0.0]);
    assert!(dodge_impulse.local_forward_component < 0.72);
    assert!(dodge_impulse.local_right_component > 0.70);

    let uncompensated_candidate = ActiveFlipImpulseCandidate {
        is_team_0: true,
        start_time: 1.0,
        start_frame: 10,
        latest_time: 1.1,
        latest_frame: 12,
        start_position: glam::Vec3::ZERO,
        end_position: glam::Vec3::ZERO,
        start_velocity: glam::Vec3::new(1000.0, 0.0, 0.0),
        end_velocity: glam::Vec3::new(1200.0, 100.0, 0.0),
        local_forward: glam::Vec3::X,
        local_right: glam::Vec3::Y,
        local_up: glam::Vec3::Z,
        boost_compensation: glam::Vec3::ZERO,
        sample_count: 2,
        boost_sample_count: 0,
        onset_local_angular_velocity: glam::Vec3::ZERO,
        min_forward_z: 0.0,
        max_forward_deviation_degrees: 0.0,
        max_up_deviation_degrees: 0.0,
        min_up_z: 1.0,
        rotation_sample_count: 0,
        dodge_torque: None,
    };
    let uncompensated = FlipImpulseCalculator::candidate_event(
        &boxcars::RemoteId::Steam(1),
        uncompensated_candidate,
    );
    let uncompensated_impulse = uncompensated
        .dodge_impulse
        .as_ref()
        .expect("expected uncompensated dodge impulse");
    assert!(uncompensated_impulse.local_forward_component > dodge_impulse.local_forward_component);
}

#[test]
fn dodge_event_carries_replicated_dodge_torque() {
    let mut calculator = FlipImpulseCalculator::new();
    let live_play = LivePlayState::active_play();
    let torque = glam::Vec3::new(0.0, 2.6, 0.0);

    let mut onset_player = player(glam::Vec3::new(1000.0, 0.0, 0.0), true, false);
    onset_player.dodge_torque = Some(torque);
    calculator
        .update_parts(
            &frame(10, 1.0, 0.0),
            &PlayerFrameState {
                players: vec![onset_player],
            },
            &live_play,
        )
        .unwrap();
    calculator
        .update_parts(
            &frame(12, 1.10, 0.10),
            &PlayerFrameState {
                players: vec![player(glam::Vec3::new(1100.0, 0.0, 0.0), true, false)],
            },
            &live_play,
        )
        .unwrap();
    calculator.finalize_parts(&frame(16, 1.30, 0.10));

    let event = calculator.events().first().expect("expected dodge event");
    assert_eq!(event.dodge_torque, Some(torque.to_array()));
}
