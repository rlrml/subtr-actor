use super::*;

fn rigid_body(position: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn ball(position: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position),
    })
}

fn player(id: u64, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0,
        rigid_body: Some(rigid_body(position)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
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
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn active_gameplay() -> GameplayState {
    GameplayState {
        ball_has_been_hit: Some(true),
        current_in_game_team_player_counts: [2, 0],
        ..Default::default()
    }
}

#[test]
fn ball_depth_fractions_treat_near_ball_band_as_level() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, -100.0, 100.0);
    assert_eq!(behind, 0.0);
    assert_eq!(level, 1.0);
    assert_eq!(in_front, 0.0);
}

#[test]
fn ball_depth_fractions_split_crossing_time_across_all_three_buckets() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, -300.0, 300.0);
    assert!((behind - 0.25).abs() < 1e-6);
    assert!((level - 0.5).abs() < 1e-6);
    assert!((in_front - 0.25).abs() < 1e-6);
}

#[test]
fn ball_depth_fractions_count_boundary_point_as_in_front_not_level() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, 150.0, 150.0);
    assert_eq!(behind, 0.0);
    assert_eq!(level, 0.0);
    assert_eq!(in_front, 1.0);
}

#[test]
fn positioning_events_emit_state_change_spans() {
    let mut calculator = PositioningCalculator::new();
    let gameplay = active_gameplay();
    let ball = ball(glam::Vec3::ZERO);
    let players = PlayerFrameState {
        players: vec![
            player(1, true, glam::vec3(0.0, -1000.0, 0.0)),
            player(2, true, glam::vec3(0.0, 1000.0, 0.0)),
        ],
    };

    for frame_number in 0..3 {
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball,
                &players,
                &FrameEventsState::default(),
                true,
                None,
            )
            .expect("positioning update should succeed");
    }
    calculator.flush_pending_events();

    let events = calculator.events();
    assert_eq!(events.len(), 2);
    let back_event = events
        .iter()
        .find(|event| event.player == boxcars::RemoteId::Steam(1))
        .expect("back player event should be emitted");
    assert_eq!(back_event.frame, 0);
    assert_eq!(back_event.end_frame, 2);
    assert!((back_event.duration - 0.3).abs() < 1e-6);
    assert!((back_event.active_game_time - 0.3).abs() < 1e-6);
    assert!((back_event.time_most_back - 0.3).abs() < 1e-6);

    let forward_event = events
        .iter()
        .find(|event| event.player == boxcars::RemoteId::Steam(2))
        .expect("forward player event should be emitted");
    assert_eq!(forward_event.frame, 0);
    assert_eq!(forward_event.end_frame, 2);
    assert!((forward_event.duration - 0.3).abs() < 1e-6);
    assert!((forward_event.active_game_time - 0.3).abs() < 1e-6);
    assert!((forward_event.time_most_forward - 0.3).abs() < 1e-6);
}
