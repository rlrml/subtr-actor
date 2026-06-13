use super::*;

fn rigid_body(
    position: glam::Vec3,
    velocity: glam::Vec3,
    angular_velocity: glam::Vec3,
) -> boxcars::RigidBody {
    rigid_body_with_yaw(position, velocity, 0.0, angular_velocity)
}

fn rigid_body_with_yaw(
    position: glam::Vec3,
    velocity: glam::Vec3,
    yaw: f32,
    local_angular_velocity: glam::Vec3,
) -> boxcars::RigidBody {
    let rotation = glam::Quat::from_rotation_z(yaw);
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: glam_to_quat(&rotation),
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&(rotation * local_angular_velocity))),
    }
}

fn player(dodge_active: bool) -> PlayerSample {
    player_with_yaw_and_angular_velocity(dodge_active, 0.0, glam::Vec3::new(0.0, 5.0, 0.0))
}

fn player_with_yaw_and_angular_velocity(
    dodge_active: bool,
    yaw: f32,
    local_angular_velocity: glam::Vec3,
) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body_with_yaw(
            glam::Vec3::new(0.0, 0.0, 17.0),
            glam::Vec3::new(650.0, 0.0, 0.0),
            yaw,
            local_angular_velocity,
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

fn ball_at_local(yaw: f32, local_position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    let rotation = glam::Quat::from_rotation_z(yaw);
    ball(rotation * local_position, velocity)
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

fn players_with_yaw_and_angular_velocity(
    dodge_active: bool,
    yaw: f32,
    local_angular_velocity: glam::Vec3,
) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_with_yaw_and_angular_velocity(
            dodge_active,
            yaw,
            local_angular_velocity,
        )],
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
                &TouchCalculator::new(),
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
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].setup_duration >= FLICK_MIN_SETUP_SECONDS);
    assert!(calculator.events()[0].ball_speed_change >= FLICK_MIN_BALL_SPEED_CHANGE);
    assert_eq!(calculator.events()[0].kind, "other");
}

#[test]
fn counts_dodge_contact_touch_when_dodge_transition_is_late() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.1), (2, 0.2)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(false),
                &touch_state(Vec::new()),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    calculator
        .update_with_touch_classification_events(
            &frame(3, 0.3),
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(false),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.3,
                frame: 3,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &[],
            &live_play,
        )
        .unwrap();

    calculator
        .update_with_touch_classification_events(
            &FrameInfo {
                frame_number: 4,
                time: 0.35,
                dt: 0.05,
                seconds_remaining: None,
            },
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(Vec::new()),
            &[TouchClassificationEvent {
                touch_id: None,
                time: 0.3,
                frame: 3,
                sample_time: 0.3,
                sample_frame: 3,
                player: player_id.clone(),
                player_position: None,
                is_team_0: true,
                kind: "hard_hit".to_owned(),
                height_band: "ground".to_owned(),
                surface: "ground".to_owned(),
                dodge_state: "dodge".to_owned(),
                intention: "shot".to_owned(),
                first_touch: false,
                contested: false,
                role: RoleState::Unknown,
                play_depth: PlayDepthState::Unknown,
                ball_speed_change: 1350.0,
                ball_movement: None,
            }],
            &live_play,
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].player, player_id);
    assert_eq!(calculator.events()[0].dodge_time, 0.3);
    assert_eq!(calculator.events()[0].time_since_dodge, 0.0);
}

#[test]
fn rejects_late_dodge_touch_with_only_single_frame_setup() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    calculator
        .update(
            &FrameInfo {
                frame_number: 1,
                time: 0.046,
                dt: 0.046,
                seconds_remaining: None,
            },
            &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
            &players(false),
            &touch_state(Vec::new()),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    calculator
        .update_with_touch_classification_events(
            &FrameInfo {
                frame_number: 2,
                time: 0.092,
                dt: 0.046,
                seconds_remaining: None,
            },
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1800.0, 0.0, 750.0),
            ),
            &players(false),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.092,
                frame: 2,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &[],
            &live_play,
        )
        .unwrap();

    calculator
        .update_with_touch_classification_events(
            &FrameInfo {
                frame_number: 3,
                time: 0.138,
                dt: 0.046,
                seconds_remaining: None,
            },
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(1800.0, 0.0, 750.0),
            ),
            &players(true),
            &touch_state(Vec::new()),
            &[TouchClassificationEvent {
                touch_id: None,
                time: 0.092,
                frame: 2,
                sample_time: 0.092,
                sample_frame: 2,
                player: player_id,
                player_position: None,
                is_team_0: true,
                kind: "hard_hit".to_owned(),
                height_band: "ground".to_owned(),
                surface: "ground".to_owned(),
                dodge_state: "dodge".to_owned(),
                intention: "shot".to_owned(),
                first_touch: false,
                contested: false,
                role: RoleState::Unknown,
                play_depth: PlayDepthState::Unknown,
                ball_speed_change: 1800.0,
                ball_movement: None,
            }],
            &live_play,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn labels_reverse_flicks_with_backflip_pitch_forward_impulse_and_rotation_under_ball() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();
    let final_yaw = 0.35;

    for (frame_number, time, yaw) in [(1, 0.1, 0.0), (2, 0.2, 0.18), (3, 0.3, final_yaw)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball_at_local(yaw, glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players_with_yaw_and_angular_velocity(
                    frame_number == 3,
                    yaw,
                    glam::Vec3::new(0.0, 0.0, 0.0),
                ),
                &touch_state(Vec::new()),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    let rotation_at_dodge = glam::Quat::from_rotation_z(final_yaw);
    let forward_impulse = rotation_at_dodge * glam::Vec3::new(1350.0, 0.0, 520.0);
    calculator
        .update(
            &frame(4, 0.4),
            &ball_at_local(
                final_yaw,
                glam::Vec3::new(180.0, 0.0, 160.0),
                forward_impulse,
            ),
            &players_with_yaw_and_angular_velocity(
                true,
                final_yaw,
                glam::Vec3::new(0.0, -5.0, 0.0),
            ),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "reverse");
    assert_eq!(event.setup_rotation_direction, "right");
    assert!(event.setup_rotation_degrees > 0.0);
    assert!(event.backflip_pitch_rate >= REVERSE_FLICK_MIN_BACKFLIP_PITCH_RATE);
    assert!(event.local_ball_impulse[0] >= REVERSE_FLICK_MIN_FORWARD_IMPULSE);
    assert!(event.rotation_under_ball_degrees >= REVERSE_FLICK_MIN_ROTATION_UNDER_BALL_DEGREES);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "reverse")]),
        1
    );
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("setup_rotation_direction", "right")]),
        1
    );
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "other")]),
        0
    );
}

#[test]
fn labels_left_reverse_flicks_from_negative_setup_rotation() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();
    let final_yaw = -0.35;

    for (frame_number, time, yaw) in [(1, 0.1, 0.0), (2, 0.2, -0.18), (3, 0.3, final_yaw)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball_at_local(yaw, glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players_with_yaw_and_angular_velocity(
                    frame_number == 3,
                    yaw,
                    glam::Vec3::new(0.0, 0.0, 0.0),
                ),
                &touch_state(Vec::new()),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    let rotation_at_dodge = glam::Quat::from_rotation_z(final_yaw);
    let forward_impulse = rotation_at_dodge * glam::Vec3::new(1350.0, 0.0, 520.0);
    calculator
        .update(
            &frame(4, 0.4),
            &ball_at_local(
                final_yaw,
                glam::Vec3::new(180.0, 0.0, 160.0),
                forward_impulse,
            ),
            &players_with_yaw_and_angular_velocity(
                true,
                final_yaw,
                glam::Vec3::new(0.0, -5.0, 0.0),
            ),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "reverse");
    assert_eq!(event.setup_rotation_direction, "left");
    assert!(event.setup_rotation_degrees < 0.0);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[
            StatLabel::new("kind", "reverse"),
            StatLabel::new("setup_rotation_direction", "left"),
        ]),
        1
    );
}

#[test]
fn frontflip_pitch_forward_impulse_is_not_labeled_reverse() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();
    let final_yaw = 0.35;

    for (frame_number, time, yaw) in [(1, 0.1, 0.0), (2, 0.2, 0.18), (3, 0.3, final_yaw)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball_at_local(yaw, glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players_with_yaw_and_angular_velocity(
                    frame_number == 3,
                    yaw,
                    glam::Vec3::new(0.0, 0.0, 0.0),
                ),
                &touch_state(Vec::new()),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    let rotation_at_dodge = glam::Quat::from_rotation_z(final_yaw);
    let forward_impulse = rotation_at_dodge * glam::Vec3::new(1350.0, 0.0, 520.0);
    calculator
        .update(
            &frame(4, 0.4),
            &ball_at_local(
                final_yaw,
                glam::Vec3::new(180.0, 0.0, 160.0),
                forward_impulse,
            ),
            &players_with_yaw_and_angular_velocity(true, final_yaw, glam::Vec3::new(0.0, 5.0, 0.0)),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "other");
    assert_eq!(
        calculator
            .player_stats()
            .get(&player_id)
            .unwrap()
            .event_count_with_labels(&[StatLabel::new("kind", "reverse")]),
        0
    );
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
            &TouchCalculator::new(),
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
                    touch_id: None,
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
            &TouchCalculator::new(),
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
                    touch_id: None,
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }]),
                &TouchCalculator::new(),
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
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
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
                    touch_id: None,
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }]),
                &TouchCalculator::new(),
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
                touch_id: None,
                time: 0.06,
                frame: 3,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
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
                &TouchCalculator::new(),
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
            &TouchCalculator::new(),
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
                touch_id: None,
                time: 0.5,
                frame: 5,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().get(&player_id).is_none());
}
