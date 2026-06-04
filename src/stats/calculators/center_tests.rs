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

fn ball(position: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position),
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

fn touch(
    frame_number: usize,
    time: f32,
    player_id: Option<PlayerId>,
    is_team_0: bool,
) -> TouchEvent {
    TouchEvent {
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: player_id,
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

fn shot_event(
    frame_number: usize,
    time: f32,
    player: PlayerId,
    is_team_0: bool,
) -> PlayerStatEvent {
    PlayerStatEvent {
        time,
        frame: frame_number,
        player,
        player_position: None,
        is_team_0,
        kind: PlayerStatEventKind::Shot,
        shot: None,
    }
}

fn goal_event(
    frame_number: usize,
    time: f32,
    player: Option<PlayerId>,
    is_team_0: bool,
) -> GoalEvent {
    GoalEvent {
        time,
        frame: frame_number,
        scoring_team_is_team_0: is_team_0,
        player,
        player_position: None,
        team_zero_score: None,
        team_one_score: None,
    }
}

fn update(
    calculator: &mut CenterCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
) {
    update_with_events(
        calculator,
        frame,
        ball,
        touch_events,
        FrameEventsState::default(),
    );
}

fn update_with_events(
    calculator: &mut CenterCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
    frame_events: FrameEventsState,
) {
    calculator
        .update(
            &frame,
            &ball,
            &TouchState {
                touch_events,
                ..TouchState::default()
            },
            &frame_events,
            &LivePlayState::active_play(),
        )
        .unwrap();
}

#[test]
fn counts_wide_touch_that_moves_ball_into_central_attacking_lane() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player.clone()), true)],
    );
    update(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.team_zero_stats().count, 1);
    assert_eq!(calculator.team_one_stats().count, 0);
    assert_eq!(calculator.player_stats().get(&player).unwrap().count, 1);
    assert_eq!(calculator.events()[0].lateral_centering_distance, 1700.0);
    assert_eq!(calculator.events()[0].ball_advance_distance, 400.0);
}

#[test]
fn ignores_ball_that_stays_wide() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player), true)],
    );
    update(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(2100.0, 3100.0, BALL_RADIUS_Z)),
        vec![],
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}

#[test]
fn ignores_touch_reported_as_shot() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update_with_events(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player.clone()), true)],
        FrameEventsState {
            player_stat_events: vec![shot_event(10, 1.0, player, true)],
            ..FrameEventsState::default()
        },
    );
    update(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}

#[test]
fn pending_center_is_cancelled_by_later_shot_event() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player.clone()), true)],
    );
    update_with_events(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
        FrameEventsState {
            player_stat_events: vec![shot_event(20, 1.8, player, true)],
            ..FrameEventsState::default()
        },
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}

#[test]
fn pending_center_is_cancelled_by_goal_event() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player.clone()), true)],
    );
    update_with_events(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
        FrameEventsState {
            goal_events: vec![goal_event(20, 1.8, Some(player), true)],
            ..FrameEventsState::default()
        },
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}

#[test]
fn pending_center_is_cancelled_by_own_goal_event() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player.clone()), true)],
    );
    update_with_events(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
        FrameEventsState {
            goal_events: vec![goal_event(20, 1.8, Some(player), false)],
            ..FrameEventsState::default()
        },
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}

#[test]
fn ignores_touch_from_defensive_half() {
    let player = PlayerId::Steam(1);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, -500.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(player), true)],
    );
    update(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 2500.0, BALL_RADIUS_Z)),
        vec![],
    );

    assert!(calculator.events().is_empty());
}

#[test]
fn opponent_touch_replaces_pending_center() {
    let passer = PlayerId::Steam(1);
    let opponent = PlayerId::Steam(2);
    let mut calculator = CenterCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(glam::Vec3::new(2600.0, 2600.0, BALL_RADIUS_Z)),
        vec![touch(10, 1.0, Some(passer), true)],
    );
    update(
        &mut calculator,
        frame(15, 1.4),
        ball(glam::Vec3::new(2200.0, 2800.0, BALL_RADIUS_Z)),
        vec![touch(15, 1.4, Some(opponent), false)],
    );
    update(
        &mut calculator,
        frame(20, 1.8),
        ball(glam::Vec3::new(900.0, 3000.0, BALL_RADIUS_Z)),
        vec![],
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().count, 0);
}
