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
        ..Default::default()
    }
}

fn touch_state(frame_number: usize, player_id: &PlayerId) -> TouchState {
    TouchState {
        touch_events: vec![TouchEvent {
            touch_id: None,
            time: frame_number as f32 * 0.1,
            frame: frame_number,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
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
            touch_id: None,
            time: frame_number as f32 * 0.1,
            frame: frame_number,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
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
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &PlayerFrameState::default(),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, true),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, false),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &dodge_contact_touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.dodge_touch_count(), 1);
    assert_eq!(stats.dodge_hit_count(), 1);
    assert_eq!(calculator.events()[0].dodge_state, "dodge");
}

#[test]
fn dodge_byte_lagging_one_frame_upgrades_touch_to_dodge() {
    // Regression: the CarComponent_Dodge ReplicatedActive byte routinely
    // replicates a frame after the ball-hit it produced, so a flip-into-ball
    // contact is sampled on the one frame where the flag is not yet active. The
    // touch must still be recognized as a dodge contact once the byte flips on
    // within the lag-tolerance window.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    // The hit lands while the dodge byte still reads inactive.
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 1500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, false),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    assert_eq!(calculator.events()[0].dodge_state, "no_dodge");
    // One frame later the dodge byte flips on (no new touch this frame).
    calculator
        .update(
            &frame(2),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 1500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, true),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events()[0].dodge_state, "dodge");
    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.dodge_touch_count(), 1);
}

#[test]
fn dodge_byte_outside_tolerance_window_does_not_upgrade_touch() {
    // A dodge that activates well after the touch (beyond the lag tolerance) is
    // a separate maneuver and must not retroactively mark the touch a dodge.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();

    calculator
        .update(
            &frame(0),
            &ball(0.0, 0.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 1500.0, 0.0)),
            &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, false),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    // Dodge stays inactive through frame 2, then activates at frame 3 — by which
    // point the touch (t=0.1) is 0.2s old, past the tolerance window.
    for (frame_number, dodge_active) in [(2usize, false), (3usize, true)] {
        calculator
            .update(
                &frame(frame_number),
                &ball_with_velocity(0.0, 0.0, glam::Vec3::new(0.0, 1500.0, 0.0)),
                &player_frame_state_with_dodge_active(&player_id, glam::Vec3::ZERO, dodge_active),
                &vertical_state(&player_id, 0.0),
                &RotationCalculator::default(),
                &TouchState::default(),
                &PossessionState::default(),
                &FiftyFiftyState::default(),
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }

    assert_eq!(calculator.events()[0].dodge_state, "no_dodge");
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
            &RotationCalculator::default(),
            &TouchState::default(),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
    let initial_touch_state = touch_state(1, &player_id);
    let followup_touch_state = TouchState {
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
            &RotationCalculator::default(),
            &initial_touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(40.0, 70.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
    let initial_touch_state = touch_state(1, &player_id);
    let followup_touch_state = TouchState {
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
            &RotationCalculator::default(),
            &initial_touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 160.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &possession(&player_id, true),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
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
    let initial_touch_state = TouchState {
        touch_events: vec![
            TouchEvent {
                touch_id: None,
                time: 0.1,
                frame: 1,
                team_is_team_0: true,
                player: Some(blue_player.clone()),
                player_position: None,
                closest_approach_distance: None,
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
                dodge_contact: false,
            },
            TouchEvent {
                touch_id: None,
                time: 0.1,
                frame: 1,
                team_is_team_0: false,
                player: Some(orange_player.clone()),
                player_position: None,
                closest_approach_distance: None,
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
                dodge_contact: false,
            },
        ],
        last_touch_player: Some(orange_player.clone()),
        last_touch_team_is_team_0: Some(false),
        ..TouchState::default()
    };
    let followup_touch_state = TouchState {
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
            &RotationCalculator::default(),
            &initial_touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty.clone()),
                ..FiftyFiftyState::default()
            },
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2),
            &ball(0.0, 100.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                active_event: Some(active_fifty),
                ..FiftyFiftyState::default()
            },
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3),
            &ball(0.0, 170.0),
            &PlayerFrameState::default(),
            &PlayerVerticalState::default(),
            &RotationCalculator::default(),
            &followup_touch_state,
            &PossessionState::default(),
            &FiftyFiftyState {
                resolved_events: vec![resolved_fifty],
                ..FiftyFiftyState::default()
            },
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let blue_stats = calculator.player_stats().get(&blue_player).unwrap();
    assert_eq!(blue_stats.total_ball_travel_distance, 170.0);
    assert_eq!(blue_stats.total_ball_advance_distance, 170.0);
    assert_eq!(blue_stats.total_ball_retreat_distance, 0.0);
    let orange_stats = calculator.player_stats().get(&orange_player).unwrap();
    assert_eq!(orange_stats.total_ball_travel_distance, 0.0);
    assert_eq!(orange_stats.total_ball_advance_distance, 0.0);
    assert_eq!(orange_stats.total_ball_retreat_distance, 0.0);
}

fn rotation_player(player_id: PlayerId, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
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

#[test]
fn touch_events_capture_rotation_role_and_play_depth() {
    let toucher = boxcars::RemoteId::Steam(1);
    let teammate = boxcars::RemoteId::Steam(2);
    let mut rotation = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_debounce_seconds: 0.0,
        ..RotationCalculatorConfig::default()
    });
    let players = PlayerFrameState {
        players: vec![
            rotation_player(toucher.clone(), true, glam::Vec3::new(0.0, -2000.0, 17.0)),
            rotation_player(teammate.clone(), true, glam::Vec3::new(0.0, -300.0, 17.0)),
            rotation_player(
                boxcars::RemoteId::Steam(3),
                false,
                glam::Vec3::new(3000.0, 3000.0, 17.0),
            ),
            rotation_player(
                boxcars::RemoteId::Steam(4),
                false,
                glam::Vec3::new(-3000.0, 3000.0, 17.0),
            ),
        ],
    };
    let gameplay = GameplayState {
        ball_has_been_hit: Some(true),
        current_in_game_team_player_counts: [2, 2],
        ..GameplayState::default()
    };
    rotation
        .update(
            &frame(1),
            &gameplay,
            &ball(0.0, 0.0),
            &players,
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    assert_eq!(
        rotation.current_role_and_depth(&toucher),
        (RoleState::SecondMan, PlayDepthState::BehindPlay)
    );

    let mut calculator = TouchCalculator::new();
    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &players,
            &vertical_state(&toucher, 0.0),
            &rotation,
            &touch_state(1, &toucher),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = &calculator.events()[0];
    assert_eq!(event.role, RoleState::SecondMan);
    assert_eq!(event.play_depth, PlayDepthState::BehindPlay);

    let stats = calculator.player_stats().get(&toucher).unwrap();
    assert_eq!(stats.touches_as_second_man(), 1);
    assert_eq!(stats.touches_as_first_man(), 0);
    assert_eq!(stats.touches_as_third_man(), 0);
    assert_eq!(stats.touches_behind_play(), 1);
    assert_eq!(stats.touches_ahead_of_play(), 0);
}

#[test]
fn touch_with_default_rotation_state_records_unknown_role_and_depth() {
    let stats = touch_stats_at_height(0.0);

    assert_eq!(stats.touch_count, 1);
    assert_eq!(stats.touch_count_with_role(RoleState::Unknown), 1);
    assert_eq!(
        stats.touch_count_with_play_depth(PlayDepthState::Unknown),
        1
    );
    assert_eq!(stats.touches_as_first_man(), 0);
    assert_eq!(stats.touches_as_second_man(), 0);
    assert_eq!(stats.touches_as_third_man(), 0);
}

#[test]
fn touch_classification_events_follow_chronological_touch_event_order() {
    let player = boxcars::RemoteId::Steam(1);
    let early_touch = TouchEvent {
        touch_id: None,
        time: 0.1,
        frame: 1,
        team_is_team_0: true,
        player: Some(player.clone()),
        player_position: None,
        closest_approach_distance: Some(2.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    };
    let late_touch = TouchEvent {
        touch_id: None,
        time: 0.4,
        frame: 4,
        team_is_team_0: true,
        player: Some(player.clone()),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
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
            &RotationCalculator::default(),
            &touch_state,
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let events = calculator.new_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].frame, 1);
    assert_eq!(events[1].frame, 4);
}

#[test]
fn soft_touch_followed_by_staying_with_ball_upgrades_intention_to_control() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();
    let followup_touch_state = TouchState {
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    };

    calculator
        .update(
            &frame(1),
            &ball(0.0, 0.0),
            &player_frame_state(&player_id, glam::Vec3::new(0.0, 0.0, 17.0)),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events()[0].intention, "neutral");
    assert!(calculator.events()[0].first_touch);

    // Stay glued to the (stationary) ball until the control-follow window
    // ages out and resolves.
    for frame_number in 2..=15 {
        calculator
            .update(
                &frame(frame_number),
                &ball(0.0, 0.0),
                &player_frame_state(&player_id, glam::Vec3::new(0.0, 0.0, 17.0)),
                &vertical_state(&player_id, 0.0),
                &RotationCalculator::default(),
                &followup_touch_state,
                &PossessionState::default(),
                &FiftyFiftyState::default(),
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }

    assert_eq!(calculator.events()[0].intention, "control");
    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.intention_count("control"), 1);
    assert_eq!(stats.first_touch_intention_count("control"), 1);
    assert_eq!(stats.first_touch_count, 1);
}

#[test]
fn touch_where_ball_leaves_the_player_stays_neutral() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = TouchCalculator::new();
    let followup_touch_state = TouchState {
        last_touch_player: Some(player_id.clone()),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    };

    calculator
        .update(
            &frame(1),
            &ball_with_velocity(0.0, 0.0, glam::Vec3::new(1000.0, 0.0, 0.0)),
            &player_frame_state(&player_id, glam::Vec3::new(0.0, -100.0, 17.0)),
            &vertical_state(&player_id, 0.0),
            &RotationCalculator::default(),
            &touch_state(1, &player_id),
            &PossessionState::default(),
            &FiftyFiftyState::default(),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    // The ball races away while the player stays put.
    for frame_number in 2..=15 {
        let time = frame_number as f32 * 0.1;
        calculator
            .update(
                &frame(frame_number),
                &ball_with_velocity(time * 1000.0, 0.0, glam::Vec3::new(1000.0, 0.0, 0.0)),
                &player_frame_state(&player_id, glam::Vec3::new(0.0, -100.0, 17.0)),
                &vertical_state(&player_id, 0.0),
                &RotationCalculator::default(),
                &followup_touch_state,
                &PossessionState::default(),
                &FiftyFiftyState::default(),
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }

    assert_eq!(calculator.events()[0].intention, "neutral");
    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.intention_count("control"), 0);
    assert_eq!(stats.intention_count("neutral"), 1);
}
