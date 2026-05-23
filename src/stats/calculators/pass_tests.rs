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
        closest_approach_distance: Some(0.0),
    }
}

fn update(
    calculator: &mut PassCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
) {
    calculator
        .update(
            &frame,
            &ball,
            &TouchState {
                touch_events,
                ..TouchState::default()
            },
            true,
        )
        .unwrap();
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
