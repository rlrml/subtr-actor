use super::*;

fn rigid_body(
    position: glam::Vec3,
    velocity: glam::Vec3,
    angular_velocity: glam::Vec3,
) -> boxcars::RigidBody {
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
        angular_velocity: Some(glam_to_vec(&angular_velocity)),
    }
}

fn player(dodge_active: bool) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        rigid_body: Some(rigid_body(
            glam::Vec3::new(0.0, 0.0, 17.0),
            glam::Vec3::new(650.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 5.0, 0.0),
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

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity, glam::Vec3::ZERO),
    })
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn players(dodge_active: bool) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player(dodge_active)],
    }
}

fn touch_state(touch_events: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events,
        last_touch_player: Some(boxcars::RemoteId::Steam(1)),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    }
}

fn live_play() -> LivePlayState {
    LivePlayState {
        is_live_play: true,
        ..LivePlayState::default()
    }
}

#[test]
fn counts_controlled_dodge_touch_with_large_ball_impulse() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(frame_number == 3),
                &touch_state(Vec::new()),
                &live_play,
            )
            .unwrap();
    }

    calculator
        .update(
            &frame(4, 0.4),
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(vec![TouchEvent {
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &live_play,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].setup_duration >= FLICK_MIN_SETUP_SECONDS);
    assert!(calculator.events()[0].ball_speed_change >= FLICK_MIN_BALL_SPEED_CHANGE);
}

#[test]
fn rejects_dodge_touch_without_controlled_setup() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    calculator
        .update(
            &frame(1, 0.1),
            &ball(glam::Vec3::new(600.0, 0.0, 112.0), glam::Vec3::ZERO),
            &players(true),
            &TouchState::default(),
            &live_play,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball(
                glam::Vec3::new(700.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &TouchState {
                touch_events: vec![TouchEvent {
                    time: 0.2,
                    frame: 2,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }],
                ..TouchState::default()
            },
            &live_play,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn setup_with_multiple_control_touches_can_count_after_minimum_duration() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(frame_number == 3),
                &touch_state(vec![TouchEvent {
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }]),
                &live_play,
            )
            .unwrap();
    }

    calculator
        .update(
            &frame(4, 0.4),
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(vec![TouchEvent {
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &live_play,
        )
        .unwrap();

    let event = calculator.events().first().unwrap();
    assert_eq!(event.setup_touch_count, 3);
    assert_eq!(calculator.player_stats().get(&player_id).unwrap().count, 1);
}

#[test]
fn rejects_tiny_multi_touch_setup() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.02), (2, 0.04)] {
        calculator
            .update(
                &FrameInfo {
                    frame_number,
                    time,
                    dt: 0.02,
                    seconds_remaining: None,
                },
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(frame_number == 2),
                &touch_state(vec![TouchEvent {
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }]),
                &live_play,
            )
            .unwrap();
    }

    calculator
        .update(
            &FrameInfo {
                frame_number: 3,
                time: 0.06,
                dt: 0.02,
                seconds_remaining: None,
            },
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(vec![TouchEvent {
                time: 0.06,
                frame: 3,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &live_play,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().get(&player_id).is_none());
}

#[test]
fn rejects_dodge_after_ball_has_left_car() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(false),
                &touch_state(Vec::new()),
                &live_play,
            )
            .unwrap();
    }

    calculator
        .update(
            &frame(4, 0.4),
            &ball(glam::Vec3::new(600.0, 0.0, 112.0), glam::Vec3::ZERO),
            &players(true),
            &touch_state(Vec::new()),
            &live_play,
        )
        .unwrap();
    calculator
        .update(
            &frame(5, 0.5),
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(vec![TouchEvent {
                time: 0.5,
                frame: 5,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &live_play,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().get(&player_id).is_none());
}
