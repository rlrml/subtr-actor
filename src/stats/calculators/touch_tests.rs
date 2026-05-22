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

fn ball(x: f32, y: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(x, y, BALL_RADIUS_Z), glam::Vec3::ZERO),
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

fn possession(player_id: &PlayerId, is_team_0: bool) -> PossessionState {
    PossessionState {
        active_team_before_sample: Some(is_team_0),
        current_team_is_team_0: Some(is_team_0),
        active_player_before_sample: Some(player_id.clone()),
        current_player: Some(player_id.clone()),
    }
}

#[test]
fn credits_ball_travel_and_goal_advancement_to_possession_player() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();
    let touch_state = TouchState {
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    };

    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(40.0, 70.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.total_ball_travel_distance, 150.0);
    assert_eq!(stats.total_ball_advance_distance, 100.0);
    assert_eq!(stats.total_ball_retreat_distance, 30.0);
}

#[test]
fn skips_ball_movement_without_a_possession_player() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();
    let touch_state = TouchState {
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    };

    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 160.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.total_ball_travel_distance, 60.0);
    assert_eq!(stats.total_ball_advance_distance, 60.0);
}

#[test]
fn credits_fifty_fifty_direction_to_resolved_winner_not_last_touch() {
    let blue_player = boxcars::RemoteId::Steam(1);
    let orange_player = boxcars::RemoteId::Steam(2);
    let mut calculator = TouchCalculator::new();
    let touch_state = TouchState {
        last_touch_player: Some(orange_player.clone()),
        last_touch_team_is_team_0: Some(false),
        ..TouchState::default()
    };

    let active_fifty = ActiveFiftyFifty {
        start_time: 0.1,
        start_frame: 1,
        last_touch_time: 0.1,
        last_touch_frame: 1,
        is_kickoff: false,
        team_zero_player: Some(blue_player.clone()),
        team_one_player: Some(orange_player.clone()),
        team_zero_position: [0.0, -100.0, 0.0],
        team_one_position: [0.0, 100.0, 0.0],
        midpoint: [0.0, 0.0, 0.0],
        plane_normal: [0.0, 1.0, 0.0],
    };
    let resolved_fifty = FiftyFiftyEvent {
        start_time: active_fifty.start_time,
        start_frame: active_fifty.start_frame,
        resolve_time: 0.3,
        resolve_frame: 3,
        is_kickoff: false,
        team_zero_player: active_fifty.team_zero_player.clone(),
        team_one_player: active_fifty.team_one_player.clone(),
        team_zero_position: active_fifty.team_zero_position,
        team_one_position: active_fifty.team_one_position,
        midpoint: active_fifty.midpoint,
        plane_normal: active_fifty.plane_normal,
        winning_team_is_team_0: Some(true),
        possession_team_is_team_0: None,
    };

    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty.clone()),
                ..FiftyFiftyState::default()
            },
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty),
                ..FiftyFiftyState::default()
            },
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 170.0),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                resolved_events: vec![resolved_fifty],
                ..FiftyFiftyState::default()
            },
            true,
        )
        .unwrap();

    let blue_stats = calculator.player_stats().get(&blue_player).unwrap();
    assert_eq!(blue_stats.total_ball_travel_distance, 170.0);
    assert_eq!(blue_stats.total_ball_advance_distance, 170.0);
    assert_eq!(blue_stats.total_ball_retreat_distance, 0.0);
    assert!(calculator.player_stats().get(&orange_player).is_none());
}
