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

fn ball(z: f32, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, z), velocity),
    })
}

fn frame(frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn player(player_id: PlayerId, z: f32, dodge_active: bool) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0: true,
        rigid_body: Some(rigid_body(glam::Vec3::new(0.0, 0.0, z), glam::Vec3::ZERO)),
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

fn players(player_id: PlayerId, z: f32, dodge_active: bool) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player(player_id, z, dodge_active)],
    }
}

fn touch(frame_number: usize, player_id: PlayerId) -> TouchEvent {
    TouchEvent {
        time: frame_number as f32 * 0.1,
        frame: frame_number,
        team_is_team_0: true,
        player: Some(player_id),
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

#[test]
fn records_half_volley_touch_after_floor_bounce() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfVolleyCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball(BALL_RADIUS_Z + 200.0, glam::Vec3::new(0.0, 0.0, -900.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(BALL_RADIUS_Z + 5.0, glam::Vec3::new(0.0, 0.0, 400.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 25.0, true),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(4),
            &ball(BALL_RADIUS_Z + 80.0, glam::Vec3::new(0.0, 1600.0, 100.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 80.0, true),
            &TouchState {
                touch_events: vec![touch(4, player_id.clone())],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    let event = calculator.events().first().expect("half-volley event");
    assert_eq!(event.player, player_id.clone());
    assert_eq!(event.bounce_frame, 2);
    assert_eq!(event.frame, 4);
    assert_eq!(calculator.team_zero_stats().count, 1);
    assert_eq!(calculator.player_stats().get(&player_id).unwrap().count, 1);
}

#[test]
fn rejects_slow_post_bounce_touches() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfVolleyCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball(BALL_RADIUS_Z + 200.0, glam::Vec3::new(0.0, 0.0, -900.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(BALL_RADIUS_Z + 5.0, glam::Vec3::new(0.0, 0.0, 400.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 25.0, true),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(BALL_RADIUS_Z + 60.0, glam::Vec3::new(0.0, 800.0, 50.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 80.0, true),
            &TouchState {
                touch_events: vec![touch(3, player_id)],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_stale_bounces() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfVolleyCalculator::with_config(HalfVolleyCalculatorConfig {
        max_bounce_to_touch_seconds: 0.2,
        ..HalfVolleyCalculatorConfig::default()
    });

    calculator
        .update(
            &frame(1),
            &ball(BALL_RADIUS_Z + 200.0, glam::Vec3::new(0.0, 0.0, -900.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(BALL_RADIUS_Z + 5.0, glam::Vec3::new(0.0, 0.0, 400.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 25.0, true),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(6),
            &ball(BALL_RADIUS_Z + 60.0, glam::Vec3::new(0.0, 1600.0, 50.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 80.0, true),
            &TouchState {
                touch_events: vec![touch(6, player_id)],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_post_bounce_touch_without_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfVolleyCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball(BALL_RADIUS_Z + 200.0, glam::Vec3::new(0.0, 0.0, -900.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(BALL_RADIUS_Z + 5.0, glam::Vec3::new(0.0, 0.0, 400.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 25.0, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(4),
            &ball(BALL_RADIUS_Z + 80.0, glam::Vec3::new(0.0, 1600.0, 100.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 80.0, false),
            &TouchState {
                touch_events: vec![touch(4, player_id)],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_dodge_without_recent_ground_contact() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = HalfVolleyCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball(BALL_RADIUS_Z + 200.0, glam::Vec3::new(0.0, 0.0, -900.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 120.0, false),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(BALL_RADIUS_Z + 5.0, glam::Vec3::new(0.0, 0.0, 400.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 120.0, true),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(4),
            &ball(BALL_RADIUS_Z + 80.0, glam::Vec3::new(0.0, 1600.0, 100.0)),
            &players(player_id.clone(), PLAYER_GROUND_Z_THRESHOLD + 120.0, true),
            &TouchState {
                touch_events: vec![touch(4, player_id)],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}
