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

fn player(player_id: PlayerId, dodge_active: bool) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(glam::Vec3::new(0.0, 0.0, 100.0))),
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

fn players(player_id: PlayerId, dodge_active: bool) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player(player_id, dodge_active)],
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 180.0)),
    })
}

fn reset_event(player: PlayerId) -> DodgeRefreshedEvent {
    DodgeRefreshedEvent {
        time: 1.0,
        frame: 10,
        player,
        is_team_0: true,
        player_position: None,
        counter_value: 1,
    }
}

fn touch_event(player: PlayerId, time: f32, frame: usize) -> TouchEvent {
    TouchEvent {
        time,
        frame,
        team_is_team_0: true,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

fn raw_team_touch_event(time: f32, frame: usize) -> TouchEvent {
    TouchEvent {
        time,
        frame,
        team_is_team_0: true,
        player: None,
        player_position: None,
        closest_approach_distance: None,
        dodge_contact: false,
    }
}

fn touch_state(touch_events: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events,
        ..TouchState::default()
    }
}

#[test]
fn on_ball_reset_alone_is_not_confirmed_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    calculator
        .update(
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState {
                dodge_refreshed_events: vec![reset_event(player_id.clone())],
                ..FrameEventsState::default()
            },
            &TouchState::default(),
        )
        .unwrap();

    assert_eq!(calculator.player_stats()[&player_id].on_ball_count, 1);
    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn touch_after_reset_requires_dodge_to_confirm_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    calculator
        .update(
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState {
                dodge_refreshed_events: vec![reset_event(player_id.clone())],
                ..FrameEventsState::default()
            },
            &TouchState::default(),
        )
        .unwrap();
    calculator
        .update(
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState {
                touch_events: vec![raw_team_touch_event(1.2, 12)],
                ..FrameEventsState::default()
            },
            &touch_state(vec![touch_event(player_id.clone(), 1.2, 12)]),
        )
        .unwrap();

    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn raw_replay_touch_after_reset_does_not_confirm_without_attributed_touch_state() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    calculator
        .update(
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState {
                dodge_refreshed_events: vec![reset_event(player_id.clone())],
                ..FrameEventsState::default()
            },
            &TouchState::default(),
        )
        .unwrap();
    calculator
        .update(
            &ball(),
            &players(player_id, true),
            &FrameEventsState::default(),
            &TouchState::default(),
        )
        .unwrap();
    calculator
        .update(
            &ball(),
            &players(boxcars::RemoteId::Steam(1), true),
            &FrameEventsState {
                touch_events: vec![raw_team_touch_event(1.3, 13)],
                ..FrameEventsState::default()
            },
            &TouchState::default(),
        )
        .unwrap();

    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn dodge_touch_after_on_ball_reset_confirms_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    calculator
        .update(
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState {
                dodge_refreshed_events: vec![reset_event(player_id.clone())],
                ..FrameEventsState::default()
            },
            &TouchState::default(),
        )
        .unwrap();
    calculator
        .update(
            &ball(),
            &players(player_id.clone(), true),
            &FrameEventsState::default(),
            &TouchState::default(),
        )
        .unwrap();
    calculator
        .update(
            &ball(),
            &players(player_id.clone(), true),
            &FrameEventsState {
                touch_events: vec![raw_team_touch_event(1.3, 13)],
                ..FrameEventsState::default()
            },
            &touch_state(vec![touch_event(player_id.clone(), 1.3, 13)]),
        )
        .unwrap();

    let event = calculator.confirmed_flip_reset_events().first().unwrap();
    assert_eq!(event.player, player_id);
    assert_eq!(event.reset_frame, 10);
    assert_eq!(event.frame, 13);
    assert!((event.time_since_reset - 0.3).abs() < 1e-5);
}
