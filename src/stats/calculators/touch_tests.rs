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
            true,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.total_ball_travel_distance, 60.0);
    assert_eq!(stats.total_ball_advance_distance, 60.0);
}
