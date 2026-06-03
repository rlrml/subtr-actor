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

fn ball(y: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, BALL_RADIUS_Z)),
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

fn update(
    calculator: &mut PassCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
) {
    update_with_context(
        calculator,
        frame,
        ball,
        touch_events,
        BackboardBounceState::default(),
        FiftyFiftyState::default(),
    );
}

fn update_with_context(
    calculator: &mut PassCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
    backboard_bounce_state: BackboardBounceState,
    fifty_fifty_state: FiftyFiftyState,
) {
    calculator
        .update(
            &frame,
            &ball,
            &TouchState {
                touch_events,
                ..TouchState::default()
            },
            &backboard_bounce_state,
            &fifty_fifty_state,
            true,
        )
        .unwrap();
}

fn backboard_bounce(
    frame_number: usize,
    time: f32,
    player: PlayerId,
    is_team_0: bool,
) -> BackboardBounceState {
    let event = BackboardBounceEvent {
        time,
        frame: frame_number,
        player,
        player_position: None,
        is_team_0,
    };
    BackboardBounceState {
        bounce_events: vec![event.clone()],
        last_bounce_event: Some(event),
    }
}

fn active_fifty_fifty(
    frame_number: usize,
    time: f32,
    team_zero_player: PlayerId,
    team_one_player: PlayerId,
) -> FiftyFiftyState {
    FiftyFiftyState {
        active_event: Some(ActiveFiftyFifty {
            start_time: time,
            start_frame: frame_number,
            last_touch_time: time,
            last_touch_frame: frame_number,
            is_kickoff: false,
            team_zero_player: Some(team_zero_player),
            team_one_player: Some(team_one_player),
            team_zero_touch_time: Some(time),
            team_zero_touch_frame: Some(frame_number),
            team_zero_dodge_contact: false,
            team_one_touch_time: Some(time),
            team_one_touch_frame: Some(frame_number),
            team_one_dodge_contact: false,
            team_zero_position: [0.0, 0.0, 0.0],
            team_one_position: [100.0, 0.0, 0.0],
            midpoint: [50.0, 0.0, 0.0],
            plane_normal: [1.0, 0.0, 0.0],
        }),
        resolved_events: Vec::new(),
        last_resolved_event: None,
    }
}

#[test]
fn counts_completed_teammate_pass_after_meaningful_ball_travel() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer.clone()), true)],
    );
    update(
        &mut calculator,
        frame(20, 2.0),
        ball(800.0),
        vec![touch(20, 2.0, Some(receiver.clone()), true)],
    );

    let passer_stats = calculator.player_stats().get(&passer).unwrap();
    let receiver_stats = calculator.player_stats().get(&receiver).unwrap();
    assert_eq!(passer_stats.completed_pass_count, 1);
    assert_eq!(receiver_stats.received_pass_count, 1);
    assert_eq!(calculator.team_zero_stats().completed_pass_count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].ball_advance_distance, 800.0);
    assert_eq!(calculator.events()[0].pass_kind, PassKind::Direct);
}

#[test]
fn ignores_short_teammate_touches() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer), true)],
    );
    update(
        &mut calculator,
        frame(11, 1.1),
        ball(120.0),
        vec![touch(11, 1.1, Some(receiver), true)],
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().completed_pass_count, 0);
}

#[test]
fn opponent_touch_breaks_pass_chain() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let opponent = PlayerId::Steam(3);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer), true)],
    );
    update(
        &mut calculator,
        frame(15, 1.5),
        ball(500.0),
        vec![touch(15, 1.5, Some(opponent), false)],
    );
    update(
        &mut calculator,
        frame(20, 2.0),
        ball(1000.0),
        vec![touch(20, 2.0, Some(receiver), true)],
    );

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.team_zero_stats().completed_pass_count, 0);
}

#[test]
fn ignores_passes_outside_duration_window() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer), true)],
    );
    update(
        &mut calculator,
        frame(60, 5.1),
        ball(1500.0),
        vec![touch(60, 5.1, Some(receiver), true)],
    );

    assert!(calculator.events().is_empty());
}

#[test]
fn counts_slow_layoff_within_extended_duration_window() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer), true)],
    );
    update(
        &mut calculator,
        frame(45, 4.49),
        ball(1500.0),
        vec![touch(45, 4.49, Some(receiver), true)],
    );

    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn classifies_backboard_passes() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut calculator = PassCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer.clone()), true)],
    );
    update_with_context(
        &mut calculator,
        frame(20, 2.0),
        ball(900.0),
        vec![touch(20, 2.0, Some(receiver), true)],
        backboard_bounce(15, 1.5, passer, true),
        FiftyFiftyState::default(),
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].pass_kind, PassKind::Backboard);
}

#[test]
fn classifies_passes_from_fifty_fifty_touches() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let opponent = PlayerId::Steam(3);
    let mut calculator = PassCalculator::new();

    update_with_context(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer.clone()), true)],
        BackboardBounceState::default(),
        active_fifty_fifty(10, 1.0, passer, opponent),
    );
    update(
        &mut calculator,
        frame(20, 2.0),
        ball(900.0),
        vec![touch(20, 2.0, Some(receiver), true)],
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].pass_kind, PassKind::FiftyFifty);
}

#[test]
fn classifies_fifty_fifty_backboard_passes() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let opponent = PlayerId::Steam(3);
    let mut calculator = PassCalculator::new();

    update_with_context(
        &mut calculator,
        frame(10, 1.0),
        ball(0.0),
        vec![touch(10, 1.0, Some(passer.clone()), true)],
        BackboardBounceState::default(),
        active_fifty_fifty(10, 1.0, passer.clone(), opponent),
    );
    update_with_context(
        &mut calculator,
        frame(20, 2.0),
        ball(900.0),
        vec![touch(20, 2.0, Some(receiver), true)],
        backboard_bounce(15, 1.5, passer, true),
        FiftyFiftyState::default(),
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(
        calculator.events()[0].pass_kind,
        PassKind::FiftyFiftyBackboard
    );
}
