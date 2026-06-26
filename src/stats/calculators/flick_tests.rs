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
            // Setup balls in these fixtures sit at a fixed position (a stationary
            // carry), so the car shares their velocity — otherwise the carry gate
            // (ball must track the car) would read a car driving under a loose
            // ball. Car linear velocity is otherwise unused by flick detection.
            glam::Vec3::ZERO,
            yaw,
            local_angular_velocity,
        )),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
        dodge_torque: None,
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

/// A single team-zero car with an explicit linear velocity, for exercising the
/// carry gate (ball-tracks-car) against a moving car.
fn moving_players(dodge_active: bool, velocity: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![PlayerSample {
            player_id: boxcars::RemoteId::Steam(1),
            is_team_0: true,
            hitbox: default_car_hitbox(),
            rigid_body: Some(rigid_body(
                glam::Vec3::new(0.0, 0.0, 17.0),
                velocity,
                glam::Vec3::new(0.0, 5.0, 0.0),
            )),
            boost_amount: None,
            last_boost_amount: None,
            boost_active: false,
            dodge_active,
            dodge_torque: None,
            powerslide_active: false,
            match_goals: None,
            match_assists: None,
            match_saves: None,
            match_shots: None,
            match_score: None,
        }],
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

/// Horizontal run velocity shared by the car and the carried ball through a
/// flick setup, giving the dodge a well-defined travel direction (+x). The
/// dodge-direction classification decomposes `dodge_torque` against this.
const FLICK_CARRY_VELOCITY: glam::Vec3 = glam::Vec3::new(1000.0, 0.0, 0.0);

fn flick_player(dodge_active: bool, dodge_torque: Option<glam::Vec3>) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(
            glam::Vec3::new(0.0, 0.0, 17.0),
            FLICK_CARRY_VELOCITY,
            glam::Vec3::new(0.0, 5.0, 0.0),
        )),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
        dodge_torque,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn flick_players(dodge_active: bool, dodge_torque: Option<glam::Vec3>) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![flick_player(dodge_active, dodge_torque)],
    }
}

/// Run a carry-then-dodge flick: three setup frames carrying the ball on the
/// hood at [`FLICK_CARRY_VELOCITY`], with the dodge transition (carrying
/// `dodge_torque`, a world-frame flip axis) on the third frame, then a launch
/// touch whose gravity-compensated ball impulse is `launch_impulse` (added to
/// the carry velocity). Returns the resolved calculator.
fn run_flick_scenario(dodge_torque: glam::Vec3, launch_impulse: glam::Vec3) -> FlickCalculator {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        let dodging = frame_number == 3;
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), FLICK_CARRY_VELOCITY),
                &flick_players(dodging, dodging.then_some(dodge_torque)),
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
                FLICK_CARRY_VELOCITY + launch_impulse,
            ),
            &flick_players(true, Some(dodge_torque)),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();
    calculator
}

// Travel direction is +x, so the right-of-travel basis is r = (0, -1). A forward
// dodge has torque . r > 0 (torque -y); a backflip has torque . r < 0 (torque
// +y); a side dodge's torque lies along travel (±x).
const FORWARD_DODGE_TORQUE: glam::Vec3 = glam::Vec3::new(0.0, -2.6, 0.0);
const BACKFLIP_DODGE_TORQUE: glam::Vec3 = glam::Vec3::new(0.0, 2.6, 0.0);
const SIDE_DODGE_TORQUE: glam::Vec3 = glam::Vec3::new(2.6, 0.0, 0.0);
// Diagonal backflips: torque +y is backflip (fb<0), torque ±x is the side/roll
// component that decides handedness (+x = right of +x travel).
const REVERSE_RIGHT_DODGE_TORQUE: glam::Vec3 = glam::Vec3::new(1.84, 1.84, 0.0);
const REVERSE_LEFT_DODGE_TORQUE: glam::Vec3 = glam::Vec3::new(-1.84, 1.84, 0.0);

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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
                ball_position: None,
                is_team_0: true,
                tags: TouchClassificationEvent::classification_tags(
                    "hard_hit",
                    "ground",
                    "ground",
                    "dodge",
                    Some("shot"),
                    false,
                    false,
                ),
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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
                ball_position: None,
                is_team_0: true,
                tags: TouchClassificationEvent::classification_tags(
                    "hard_hit",
                    "ground",
                    "ground",
                    "dodge",
                    Some("shot"),
                    false,
                    false,
                ),
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
fn labels_forward_flick_from_forward_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    let calculator = run_flick_scenario(FORWARD_DODGE_TORQUE, glam::Vec3::new(1350.0, 0.0, 520.0));

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "forward");
    assert_eq!(event.direction, "center");
    assert!(event.dodge_forward_back > 0.0);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "forward")]),
        1
    );
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "reverse")]),
        0
    );
}

#[test]
fn labels_reverse_flick_from_backflip_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    // A pure backflip (dodge straight back) is the defining reverse flick. No
    // sideways dodge component, so it is center.
    let calculator = run_flick_scenario(BACKFLIP_DODGE_TORQUE, glam::Vec3::new(1350.0, 0.0, 520.0));

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "reverse");
    assert_eq!(event.direction, "center");
    assert!(event.dodge_forward_back <= -REVERSE_FLICK_MIN_BACKWARD);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "reverse")]),
        1
    );
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("direction", "center")]),
        1
    );
}

#[test]
fn labels_right_reverse_flick_from_rightward_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    // A reverse 45 to the right: the dodge is back + right (roll component), so
    // handedness comes from the dodge's own side, not where the ball ends up.
    let calculator = run_flick_scenario(
        REVERSE_RIGHT_DODGE_TORQUE,
        glam::Vec3::new(1350.0, 0.0, 520.0),
    );

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "reverse");
    assert_eq!(event.direction, "right");
    assert!(event.dodge_side > 0.0);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[
            StatLabel::new("kind", "reverse"),
            StatLabel::new("direction", "right"),
        ]),
        1
    );
}

#[test]
fn labels_left_reverse_flick_from_leftward_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    let calculator = run_flick_scenario(
        REVERSE_LEFT_DODGE_TORQUE,
        glam::Vec3::new(1350.0, 0.0, 520.0),
    );

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "reverse");
    assert_eq!(event.direction, "left");
    assert!(event.dodge_side < 0.0);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[
            StatLabel::new("kind", "reverse"),
            StatLabel::new("direction", "left"),
        ]),
        1
    );
}

#[test]
fn labels_side_flick_from_side_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    // A sideways (roll) dodge whose torque lies along travel: a side flick, not a
    // reverse one even though the ball still goes forward.
    let calculator = run_flick_scenario(SIDE_DODGE_TORQUE, glam::Vec3::new(1350.0, 0.0, 520.0));

    let event = calculator.events().first().unwrap();
    assert_eq!(event.kind, "side");
    assert!(event.dodge_side.abs() >= SIDE_FLICK_MIN_SIDE);

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "side")]),
        1
    );
    assert_eq!(
        stats.event_count_with_labels(&[StatLabel::new("kind", "reverse")]),
        0
    );
}

#[test]
fn forward_dodge_is_not_labeled_reverse() {
    // A forward dodge launching the ball forward is a forward flick, never a
    // reverse one — the dodge direction, not the ball going forward, decides.
    let calculator = run_flick_scenario(FORWARD_DODGE_TORQUE, glam::Vec3::new(1350.0, 0.0, 520.0));
    let event = calculator.events().first().unwrap();
    assert_ne!(event.kind, "reverse");
}

#[test]
fn rejects_dodge_into_loose_ball_that_is_not_carried() {
    // A car driving fast into a loose ball that merely passes through the control
    // volume, then dodging, is not a flick — the ball never tracks the car. This
    // is otherwise identical to `counts_controlled_dodge_touch_with_large_ball_impulse`;
    // only the ball-vs-car relative velocity differs. Regression for the
    // false-positive flick at ~52.9s in problematic-private-duel-2026-03-20,
    // where a dodge-pop off a loose ball was labeled a flick.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    // Car and loose ball differ by ~600 uu/s horizontally — well above the carry
    // threshold, so no setup frame counts as a carry.
    let car_velocity = glam::Vec3::new(1400.0, 0.0, 0.0);
    let loose_ball_velocity = glam::Vec3::new(800.0, 0.0, 0.0);

    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), loose_ball_velocity),
                &moving_players(frame_number == 3, car_velocity),
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
            &moving_players(true, car_velocity),
            &touch_state(vec![TouchEvent {
                touch_id: None,
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: Some(0.0),
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
                dodge_contact: false,
            }]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
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
                    contact_local_ball_position: None,
                    contact_local_hitbox_point: None,
                    contact_world_hitbox_point: None,
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
                    contact_local_ball_position: None,
                    contact_local_hitbox_point: None,
                    contact_world_hitbox_point: None,
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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
                    contact_local_ball_position: None,
                    contact_local_hitbox_point: None,
                    contact_world_hitbox_point: None,
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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
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
fn counts_flick_whose_ball_impulse_arrives_after_the_touch_frame() {
    // A carry / 180 flick drags the ball through the dodge: the touch is
    // detected before the ball has finished accelerating, so the single-frame
    // impulse at the touch frame is well below the flick threshold. The
    // windowed peak impulse must still recognize the flick a few frames later.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    let control_touch = |frame_number: usize, time: f32| TouchEvent {
        touch_id: None,
        time,
        frame: frame_number,
        team_is_team_0: true,
        player: Some(player_id.clone()),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    };

    // Dribble setup with the dodge transition on the third frame.
    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(frame_number == 3),
                &touch_state(vec![control_touch(frame_number, time)]),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    // Dodge touch: the ball has barely changed velocity yet, so the single-frame
    // impulse is far below `FLICK_MIN_BALL_SPEED_CHANGE`. No flick yet.
    calculator
        .update(
            &frame(4, 0.4),
            &ball(
                glam::Vec3::new(180.0, 0.0, 160.0),
                glam::Vec3::new(40.0, 0.0, 20.0),
            ),
            &players(true),
            &touch_state(vec![control_touch(4, 0.4)]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();
    assert!(calculator.events().is_empty());

    // One frame later — still inside the impulse window — the flick's power has
    // fully landed. The peak impulse now clears the gate and the flick fires,
    // exactly once.
    calculator
        .update(
            &frame(5, 0.5),
            &ball(
                glam::Vec3::new(260.0, 0.0, 205.0),
                glam::Vec3::new(1350.0, 0.0, 560.0),
            ),
            &players(true),
            &TouchState::default(),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].ball_speed_change >= FLICK_MIN_BALL_SPEED_CHANGE);
    assert_eq!(calculator.player_stats().get(&player_id).unwrap().count, 1);
}

// Builds the control touch a dribbling team-zero player taps each frame.
fn carry_touch(frame_number: usize, time: f32) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame: frame_number,
        team_is_team_0: true,
        player: Some(boxcars::RemoteId::Steam(1)),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

#[test]
fn bridges_brief_control_gaps_into_one_setup() {
    // A real carry lets the ball wobble in and out of the tight control volume.
    // Here the ball leaves the volume every other frame, so without gap bridging
    // no continuous run reaches FLICK_MIN_SETUP_SECONDS (each isolated frame is
    // only one dt) and the flick is never recognized. With bridging the observed
    // frames accumulate into one qualifying setup.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    let in_volume = || ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO);
    let out_of_volume = || ball(glam::Vec3::new(260.0, 0.0, 112.0), glam::Vec3::ZERO);

    // Observe / drop / observe / drop: isolated single-frame carries.
    for (frame_number, time, observed) in [
        (1usize, 0.1f32, true),
        (2, 0.2, false),
        (3, 0.3, true),
        (4, 0.4, false),
    ] {
        calculator
            .update(
                &frame(frame_number, time),
                &if observed {
                    in_volume()
                } else {
                    out_of_volume()
                },
                &players(false),
                &touch_state(vec![carry_touch(frame_number, time)]),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    // Dodge + flick touch on an observed frame, with the ball launched.
    calculator
        .update(
            &frame(5, 0.5),
            &ball(
                glam::Vec3::new(60.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(true),
            &touch_state(vec![carry_touch(5, 0.5)]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].setup_duration >= FLICK_MIN_SETUP_SECONDS);
    assert_eq!(calculator.player_stats().get(&player_id).unwrap().count, 1);
}

#[test]
fn counts_flick_when_dodge_byte_lags_the_contact() {
    // The ball can start accelerating a frame or two before the dodge-active byte
    // flips, so the flick touch precedes the recorded dodge start (a negative
    // `time_since_dodge`). That must not be rejected as "touch before dodge".
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = FlickCalculator::new();
    let live_play = live_play();

    // Carry setup (no dodge yet) long enough to qualify.
    for (frame_number, time) in [(1, 0.1), (2, 0.2), (3, 0.3)] {
        calculator
            .update(
                &frame(frame_number, time),
                &ball(glam::Vec3::new(60.0, 0.0, 112.0), glam::Vec3::ZERO),
                &players(false),
                &touch_state(vec![carry_touch(frame_number, time)]),
                &TouchCalculator::new(),
                &live_play,
            )
            .unwrap();
    }

    // Contact frame: the ball is launched but the dodge byte has not flipped yet.
    calculator
        .update(
            &frame(4, 0.4),
            &ball(
                glam::Vec3::new(60.0, 0.0, 160.0),
                glam::Vec3::new(1350.0, 0.0, 520.0),
            ),
            &players(false),
            &touch_state(vec![carry_touch(4, 0.4)]),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();
    assert!(calculator.events().is_empty());

    // Dodge byte flips one frame later. No fresh touch this frame, so the flick
    // stays anchored to the earlier contact and resolves despite the negative
    // time-since-dodge.
    calculator
        .update(
            &frame(5, 0.5),
            &ball(
                glam::Vec3::new(60.0, 0.0, 200.0),
                glam::Vec3::new(1350.0, 0.0, 540.0),
            ),
            &players(true),
            &touch_state(Vec::new()),
            &TouchCalculator::new(),
            &live_play,
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert!(calculator.events()[0].time_since_dodge < 0.0);
    assert_eq!(calculator.player_stats().get(&player_id).unwrap().count, 1);
}
