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
    ball_with_velocity(x, y, glam::Vec3::ZERO)
}

fn ball_with_velocity(x: f32, y: f32, velocity: glam::Vec3) -> BallFrameState {
    ball_with_position_and_velocity(glam::Vec3::new(x, y, BALL_RADIUS_Z), velocity)
}

fn ball_with_position_and_velocity(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
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

fn touch_state(frame_number: usize, player_id: &PlayerId) -> TouchState {
    TouchState {
        touch_events: vec![TouchEvent {
            time: frame_number as f32 * 0.1,
            frame: frame_number,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            dodge_contact: false,
        }],
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    }
}

fn dodge_contact_touch_state(frame_number: usize, player_id: &PlayerId) -> TouchState {
    TouchState {
        touch_events: vec![TouchEvent {
            time: frame_number as f32 * 0.1,
            frame: frame_number,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            dodge_contact: true,
        }],
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    }
}

fn vertical_state(player_id: &PlayerId, height: f32) -> PlayerVerticalState {
    PlayerVerticalState {
        players: std::collections::HashMap::from([(
            player_id.clone(),
            PlayerVerticalSample::from_height(height),
        )]),
    }
}

fn player_frame_state(player_id: &PlayerId, position: glam::Vec3) -> PlayerFrameState {
    player_frame_state_with_dodge_active(player_id, position, false)
}

fn player_frame_state_with_dodge_active(
    player_id: &PlayerId,
    position: glam::Vec3,
    dodge_active: bool,
) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![PlayerSample {
            player_id: player_id.clone(),
            is_team_0: true,
            hitbox: default_car_hitbox(),
            rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
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
        }],
    }
}

fn touch_stats_at_height(height: f32) -> TouchStats {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &vertical_state(&player_id, height),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    calculator.player_stats().get(&player_id).unwrap().clone()
}

#[test]
fn below_aerial_minimum_touch_does_not_count_as_aerial_touch() {
    let stats = touch_stats_at_height(AERIAL_TOUCH_MIN_PLAYER_Z - 1.0);

    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.aerial_touch_count, 0);
    assert_eq!(stats.high_aerial_touch_count, 0);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("height_band", "ground")]),
        1
    );
}

#[test]
fn touch_at_aerial_minimum_counts_as_aerial_touch() {
    let stats = touch_stats_at_height(AERIAL_TOUCH_MIN_PLAYER_Z);

    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.aerial_touch_count, 1);
    assert_eq!(stats.high_aerial_touch_count, 0);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("height_band", "low_air")]),
        1
    );
}

#[test]
fn uncontrolled_ground_touch_with_medium_impulse_counts_as_medium_hit() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &PlayerFrameState::default(),
            &vertical_state(&player_id, 0.0),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.control_touch_count, 0);
    assert_eq!(stats.medium_hit_count, 1);
    assert_eq!(stats.dodge_hit_count(), 0);
}

#[test]
fn dodge_active_hit_counts_as_dodge_hit() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, true),
            &vertical_state(&player_id, 0.0),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.medium_hit_count, 1);
    assert_eq!(stats.dodge_touch_count(), 1);
    assert_eq!(stats.dodge_hit_count(), 1);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("dodge_state", "dodge")]),
        1
    );
    assert_eq!(
        stats.touch_count_with_labels(&[
            StatLabel::new("kind", "medium_hit"),
            StatLabel::new("dodge_state", "dodge"),
        ]),
        1
    );
}

#[test]
fn touch_event_dodge_contact_flag_counts_as_dodge_hit() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, false),
            &vertical_state(&player_id, 0.0),
            &dodge_contact_touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.dodge_touch_count(), 1);
    assert_eq!(stats.dodge_hit_count(), 1);
    assert_eq!(calculator.events()[0].dodge_state, "dodge");
}

#[test]
fn controlled_ground_carry_touch_counts_as_control_despite_medium_impulse() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_position_and_velocity(
                glam::Vec3::new(0.0, 0.0, BALL_CARRY_MIN_BALL_Z),
                glam::Vec3::new(0.0, 500.0, 0.0),
            ),
            &player_frame_state(&player_id, glam::Vec3::new(0.0, 0.0, 17.0)),
            &vertical_state(&player_id, 0.0),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.control_touch_count, 1);
    assert_eq!(stats.medium_hit_count, 0);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("kind", "control")]),
        1
    );
}

#[test]
fn touch_on_wall_gets_wall_surface_classification() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(1),
            &ball_with_position_and_velocity(glam::Vec3::new(3720.0, 0.0, 520.0), glam::Vec3::ZERO),
            &player_frame_state(&player_id, glam::Vec3::new(3650.0, 0.0, 360.0)),
            &vertical_state(&player_id, 360.0),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.wall_touch_count, 1);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("surface", "wall")]),
        1
    );
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
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(40.0, 70.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
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
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 160.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
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
        team_zero_touch_time: Some(0.1),
        team_zero_touch_frame: Some(1),
        team_zero_dodge_contact: false,
        team_one_touch_time: Some(0.1),
        team_one_touch_frame: Some(1),
        team_one_dodge_contact: false,
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
        team_zero_touch_time: active_fifty.team_zero_touch_time,
        team_zero_touch_frame: active_fifty.team_zero_touch_frame,
        team_zero_dodge_contact: active_fifty.team_zero_dodge_contact,
        team_one_touch_time: active_fifty.team_one_touch_time,
        team_one_touch_frame: active_fifty.team_one_touch_frame,
        team_one_dodge_contact: active_fifty.team_one_dodge_contact,
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
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty.clone()),
                ..FiftyFiftyState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty),
                ..FiftyFiftyState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 170.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                resolved_events: vec![resolved_fifty],
                ..FiftyFiftyState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    let blue_stats = calculator.player_stats().get(&blue_player).unwrap();
    assert_eq!(blue_stats.total_ball_travel_distance, 170.0);
    assert_eq!(blue_stats.total_ball_advance_distance, 170.0);
    assert_eq!(blue_stats.total_ball_retreat_distance, 0.0);
    assert!(calculator.player_stats().get(&orange_player).is_none());
}

#[test]
fn touch_classification_events_follow_chronological_touch_event_order() {
    let player = boxcars::RemoteId::Steam(1);
    let early_touch = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: true,
        player: Some(player.clone()),
        player_position: None,
        closest_approach_distance: Some(2.0),
        dodge_contact: false,
    };
    let late_touch = TouchEvent {
        time: 0.4,
        frame: 4,
        team_is_team_0: true,
        player: Some(player.clone()),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    };
    let touch_state = TouchState {
        touch_events: vec![early_touch, late_touch.clone()],
        last_touch: Some(late_touch),
        last_touch_player: Some(player.clone()),
        last_touch_team_is_team_0: Some(true),
    };
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(4),
            &ball(0.0, 0.0),
            &player_frame_state(&player, glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z)),
            &PlayerVerticalState::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let events = calculator.new_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].frame, 1);
    assert_eq!(events[1].frame, 4);
}
