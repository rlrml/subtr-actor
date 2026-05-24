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
        start_speed: 900.0,
        max_speed: 1800.0,
        best_alignment: 0.98,
        best_boost_alignment: 0.98,
        boost_alignment_sample_count,
        best_diagonal_score: 1.0,
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
            false,
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
            false,
        )
        .unwrap();

    assert_eq!(
        calculator.current_kickoff_start_time,
        Some(motion_frame.time)
    );
}
