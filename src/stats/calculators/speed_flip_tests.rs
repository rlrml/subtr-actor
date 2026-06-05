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

fn player(velocity: glam::Vec3, dodge_active: bool) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(
            glam::Vec3::new(-2048.0, -2560.0, 17.0),
            velocity,
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

fn strong_candidate(boost_alignment_sample_count: u32) -> ActiveSpeedFlipCandidate {
    ActiveSpeedFlipCandidate {
        is_team_0: true,
        is_kickoff: false,
        kickoff_start_time: None,
        start_time: 1.0,
        start_frame: 10,
        start_position: [0.0, 0.0, 17.0],
        end_position: [450.0, 0.0, 50.0],
        start_velocity: glam::Vec3::new(900.0, 0.0, 0.0),
        start_velocity_xy: glam::Vec2::new(900.0, 0.0),
        start_forward_xy: glam::Vec2::X,
        local_forward: glam::Vec3::X,
        local_right: glam::Vec3::Y,
        local_up: glam::Vec3::Z,
        start_speed: 900.0,
        max_speed: 1800.0,
        best_alignment: 0.98,
        initial_boost_alignment: (boost_alignment_sample_count > 0).then_some(0.91),
        best_boost_alignment: 0.98,
        boost_alignment_sample_count,
        dodge_delay_after_ground_leave_seconds: 0.045,
        dodge_boost_compensation: glam::Vec3::ZERO,
        best_dodge_forward_delta: 320.0,
        best_dodge_delta_alignment: 0.72,
        best_estimated_dodge_impulse_magnitude: 340.0,
        best_estimated_dodge_impulse_forward_component: 0.72,
        best_estimated_dodge_impulse_side_component: 0.42,
        best_estimated_dodge_impulse_up_component: 0.08,
        dodge_acceleration_sample_count: 2,
        best_diagonal_score: 1.0,
        max_forward_rotation_degrees: 24.0,
        max_up_rotation_degrees: 120.0,
        min_forward_z: -0.16,
        latest_forward_z: 0.02,
        latest_time: 1.32,
        latest_frame: 20,
    }
}

#[test]
fn diagonal_score_accepts_side_dominant_pitch_and_side_spin() {
    let score = SpeedFlipCalculator::diagonal_score(glam::Vec3::new(64.0, 55.0, 186.0));

    assert!(
        score > 0.2,
        "expected non-zero speed-flip-like diagonal score, got {score}"
    );
    assert_eq!(
        SpeedFlipCalculator::diagonal_score(glam::Vec3::new(0.0, 0.0, 186.0)),
        0.0
    );
    assert_eq!(
        SpeedFlipCalculator::diagonal_score(glam::Vec3::new(0.0, 186.0, 0.0)),
        0.0
    );
}

#[test]
fn candidate_event_requires_boost_aligned_sample() {
    let player_id = boxcars::RemoteId::Steam(1);

    assert!(SpeedFlipCalculator::candidate_event(&player_id, strong_candidate(0)).is_none());
    assert!(SpeedFlipCalculator::candidate_event(&player_id, strong_candidate(2)).is_some());
}

#[test]
fn candidate_event_exports_alignment_and_dodge_timing_metadata() {
    let player_id = boxcars::RemoteId::Steam(1);

    let event = SpeedFlipCalculator::candidate_event(&player_id, strong_candidate(2)).unwrap();

    assert_eq!(event.initial_boost_alignment, 0.91);
    assert_eq!(event.best_boost_alignment, 0.98);
    assert_eq!(event.boost_alignment_sample_count, 2);
    assert_eq!(event.dodge_delay_after_ground_leave_seconds, 0.045);
}

#[test]
fn candidate_event_rejects_sideways_dodge_acceleration() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut sideways_candidate = strong_candidate(2);
    sideways_candidate.best_dodge_forward_delta = 50.0;
    sideways_candidate.best_dodge_delta_alignment = 0.10;

    assert!(SpeedFlipCalculator::candidate_event(&player_id, sideways_candidate).is_none());
}

#[test]
fn candidate_event_rejects_frontflip_like_candidate_without_diagonal_rotation() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut frontflip_candidate = strong_candidate(2);
    frontflip_candidate.best_estimated_dodge_impulse_forward_component = 0.98;
    frontflip_candidate.best_estimated_dodge_impulse_side_component = 0.04;
    frontflip_candidate.best_estimated_dodge_impulse_up_component = 0.10;
    frontflip_candidate.best_diagonal_score = 0.10;

    assert!(SpeedFlipCalculator::candidate_event(&player_id, frontflip_candidate).is_none());
}

#[test]
fn candidate_event_rejects_sideflip_like_impulse_without_forward_component() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut sideflip_candidate = strong_candidate(2);
    sideflip_candidate.best_estimated_dodge_impulse_forward_component = 0.12;
    sideflip_candidate.best_estimated_dodge_impulse_side_component = 0.96;
    sideflip_candidate.best_estimated_dodge_impulse_up_component = 0.10;

    assert!(SpeedFlipCalculator::candidate_event(&player_id, sideflip_candidate).is_none());
}

#[test]
fn candidate_event_rejects_vertical_dominant_wavedash_like_impulse() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut wavedash_candidate = strong_candidate(2);
    wavedash_candidate.best_estimated_dodge_impulse_forward_component = 0.45;
    wavedash_candidate.best_estimated_dodge_impulse_side_component = 0.24;
    wavedash_candidate.best_estimated_dodge_impulse_up_component = -0.86;

    assert!(SpeedFlipCalculator::candidate_event(&player_id, wavedash_candidate).is_none());
}

#[test]
fn candidate_event_rejects_wavedash_like_candidate_without_full_rotation_progress() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut wavedash_candidate = strong_candidate(2);
    wavedash_candidate.max_forward_rotation_degrees = 12.0;
    wavedash_candidate.max_up_rotation_degrees = 24.0;

    assert!(SpeedFlipCalculator::candidate_event(&player_id, wavedash_candidate).is_none());
}

#[test]
fn update_candidate_tracks_early_forward_diagonal_dodge_impulse() {
    let mut candidate = strong_candidate(1);
    candidate.start_velocity = glam::Vec3::new(900.0, 0.0, 0.0);
    candidate.start_velocity_xy = glam::Vec2::new(900.0, 0.0);
    candidate.start_forward_xy = glam::Vec2::X;
    candidate.local_forward = glam::Vec3::X;
    candidate.local_right = glam::Vec3::Y;
    candidate.local_up = glam::Vec3::Z;
    candidate.dodge_boost_compensation = glam::Vec3::ZERO;
    candidate.best_dodge_forward_delta = 0.0;
    candidate.best_dodge_delta_alignment = -1.0;
    candidate.best_estimated_dodge_impulse_magnitude = 0.0;
    candidate.best_estimated_dodge_impulse_forward_component = -1.0;
    candidate.best_estimated_dodge_impulse_side_component = 0.0;
    candidate.best_estimated_dodge_impulse_up_component = 1.0;
    candidate.dodge_acceleration_sample_count = 0;
    candidate.initial_boost_alignment = None;
    candidate.best_boost_alignment = 0.5;
    candidate.boost_alignment_sample_count = 0;
    let frame = FrameInfo {
        frame_number: 12,
        time: candidate.start_time + 0.10,
        dt: 0.05,
        seconds_remaining: None,
    };

    SpeedFlipCalculator::update_candidate(
        &mut candidate,
        &frame,
        &BallFrameState::default(),
        &PlayerSample {
            boost_active: true,
            ..player(glam::Vec3::new(1200.0, 120.0, 0.0), true)
        },
    );

    assert_eq!(candidate.dodge_acceleration_sample_count, 1);
    assert!(candidate.best_dodge_forward_delta >= 299.0);
    assert!(candidate.best_dodge_delta_alignment > 0.9);
    assert!(candidate.best_estimated_dodge_impulse_magnitude >= 270.0);
    assert!(candidate.best_estimated_dodge_impulse_forward_component > 0.9);
    assert!(candidate.best_estimated_dodge_impulse_side_component > 0.3);
    assert!(candidate.initial_boost_alignment.unwrap() > 0.99);
    assert!(candidate.best_boost_alignment > 0.99);
    assert_eq!(candidate.boost_alignment_sample_count, 1);
}

#[test]
fn kickoff_approach_waits_for_player_motion_even_when_not_live_play() {
    let mut calculator = SpeedFlipCalculator::default();
    let frame = FrameInfo {
        frame_number: 1,
        time: 0.5,
        dt: 0.1,
        seconds_remaining: None,
    };
    let gameplay = GameplayState {
        ball_has_been_hit: Some(false),
        ..Default::default()
    };

    calculator
        .update_parts(
            &frame,
            &gameplay,
            &BallFrameState::default(),
            &PlayerFrameState::default(),
            &LivePlayState::default(),
        )
        .unwrap();

    assert!(calculator.kickoff_approach_active_last_frame);
    assert_eq!(calculator.current_kickoff_start_time, None);

    let motion_frame = FrameInfo {
        frame_number: 2,
        time: 0.6,
        dt: 0.1,
        seconds_remaining: None,
    };
    calculator
        .update_parts(
            &motion_frame,
            &gameplay,
            &BallFrameState::default(),
            &PlayerFrameState {
                players: vec![player(glam::Vec3::new(150.0, 0.0, 0.0), false)],
            },
            &LivePlayState::default(),
        )
        .unwrap();

    assert_eq!(
        calculator.current_kickoff_start_time,
        Some(motion_frame.time)
    );
}
