use super::*;

fn player_id(id: u64) -> PlayerId {
    boxcars::RemoteId::Steam(id)
}

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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.25,
        seconds_remaining: None,
    }
}

fn ball(y: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, BALL_RADIUS_Z)),
    })
}

fn players(player_one_y: f32, player_two_y: f32) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![
            PlayerSample {
                player_id: player_id(1),
                is_team_0: true,
                hitbox: default_car_hitbox(),
                rigid_body: Some(rigid_body(glam::Vec3::new(0.0, player_one_y, 0.0))),
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
            },
            PlayerSample {
                player_id: player_id(2),
                is_team_0: false,
                hitbox: default_car_hitbox(),
                rigid_body: Some(rigid_body(glam::Vec3::new(0.0, player_two_y, 0.0))),
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
            },
        ],
    }
}

fn touch(frame_number: usize, time: f32, player: PlayerId, is_team_0: bool) -> TouchEvent {
    TouchEvent {
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: None,
        dodge_contact: false,
    }
}

fn touch_state(touches: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events: touches,
        ..TouchState::default()
    }
}

fn update(
    calculator: &mut ControlledPlayCalculator,
    frame_number: usize,
    time: f32,
    ball_y: f32,
    touches: Vec<TouchEvent>,
) {
    calculator
        .update(
            &frame(frame_number, time),
            &ball(ball_y),
            &players(ball_y, 4000.0),
            &touch_state(touches),
            &LivePlayState::active_play(),
        )
        .unwrap();
}

#[test]
fn emits_same_player_controlled_play_with_touch_span_and_close_time() {
    let mut calculator = ControlledPlayCalculator::new();
    update(
        &mut calculator,
        0,
        0.0,
        0.0,
        vec![touch(0, 0.0, player_id(1), true)],
    );
    update(&mut calculator, 1, 0.25, 100.0, vec![]);
    update(&mut calculator, 2, 0.50, 250.0, vec![]);
    update(&mut calculator, 3, 0.75, 500.0, vec![]);
    update(
        &mut calculator,
        4,
        1.00,
        900.0,
        vec![touch(4, 1.00, player_id(1), true)],
    );

    calculator.finish();

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.player_id, player_id(1));
    assert_eq!(event.touch_count, 2);
    assert_eq!(event.first_touch_time, 0.0);
    assert_eq!(event.last_touch_time, 1.0);
    assert_eq!(event.duration, 1.0);
    assert_eq!(event.close_duration, 1.0);
    assert_eq!(event.total_advance_distance, 900.0);
}

#[test]
fn rejects_candidate_when_first_to_last_touch_span_is_too_short() {
    let mut calculator = ControlledPlayCalculator::new();
    update(
        &mut calculator,
        0,
        0.0,
        0.0,
        vec![touch(0, 0.0, player_id(1), true)],
    );
    update(&mut calculator, 1, 0.25, 100.0, vec![]);
    update(&mut calculator, 2, 0.50, 250.0, vec![]);
    update(&mut calculator, 3, 0.75, 500.0, vec![]);
    update(
        &mut calculator,
        4,
        0.90,
        900.0,
        vec![touch(4, 0.90, player_id(1), true)],
    );
    update(&mut calculator, 5, 1.25, 900.0, vec![]);

    calculator.finish();

    assert!(calculator.events().is_empty());
}

#[test]
fn other_player_touch_breaks_candidate_before_it_can_validate() {
    let mut calculator = ControlledPlayCalculator::new();
    update(
        &mut calculator,
        0,
        0.0,
        0.0,
        vec![touch(0, 0.0, player_id(1), true)],
    );
    update(
        &mut calculator,
        1,
        0.50,
        100.0,
        vec![touch(1, 0.50, player_id(2), false)],
    );
    update(
        &mut calculator,
        2,
        1.25,
        200.0,
        vec![touch(2, 1.25, player_id(1), true)],
    );

    calculator.finish();

    assert!(calculator.events().is_empty());
}
