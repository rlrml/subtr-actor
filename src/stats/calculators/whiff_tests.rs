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

fn player_at(
    id: u64,
    position: glam::Vec3,
    velocity: glam::Vec3,
    dodge_active: bool,
) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0: true,
        rigid_body: Some(rigid_body(position, velocity)),
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

fn player(id: u64, x: f32, dodge_active: bool) -> PlayerSample {
    player_at(
        id,
        glam::Vec3::new(x, 0.0, 17.0),
        glam::Vec3::new(900.0, 0.0, 0.0),
        dodge_active,
    )
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), glam::Vec3::ZERO),
    })
}

fn moving_ball(velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), velocity),
    })
}

#[test]
fn counts_near_miss_after_player_exits_ball_area() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &touch_state,
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, 460.0, false)],
            },
            &touch_state,
            true,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.whiff_count, 1);
    assert_eq!(stats.grounded_whiff_count, 1);
    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn touch_cancels_active_whiff_candidate() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, 460.0, false)],
            },
            &TouchState {
                touch_events: vec![TouchEvent {
                    time: 0.2,
                    frame: 2,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    closest_approach_distance: Some(0.0),
                }],
                ..TouchState::default()
            },
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn lateral_drive_by_is_not_a_whiff() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-70.0, -170.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, -170.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn matching_ball_velocity_is_not_a_whiff_attempt() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = moving_ball(glam::Vec3::new(650.0, 0.0, 0.0));
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-180.0, 0.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, 0.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn side_dodge_is_not_a_whiff_attempt() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-70.0, -190.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    true,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, -190.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    true,
                )],
            },
            &touch_state,
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}
