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

fn ball(y: f32, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, BALL_RADIUS_Z), velocity),
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

fn touch(frame_number: usize, time: f32, player_id: PlayerId, is_team_0: bool) -> TouchEvent {
    TouchEvent {
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: Some(player_id),
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

fn update_pass(
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
            &BackboardBounceState::default(),
            &FiftyFiftyState::default(),
            true,
        )
        .unwrap();
}

fn update_one_timer(
    calculator: &mut OneTimerCalculator,
    pass_calculator: &PassCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
) {
    calculator
        .update(&frame, &ball, pass_calculator, true)
        .unwrap();
}

#[test]
fn counts_goal_directed_fast_receiver_touch_after_completed_pass() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut pass_calculator = PassCalculator::new();
    let mut one_timer = OneTimerCalculator::new();

    update_pass(
        &mut pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
        vec![touch(10, 1.0, passer, true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
    );

    update_pass(
        &mut pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 1600.0, 0.0)),
        vec![touch(20, 2.0, receiver.clone(), true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 1600.0, 0.0)),
    );

    let stats = one_timer.player_stats().get(&receiver).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(one_timer.team_zero_stats().count, 1);
    assert_eq!(one_timer.events().len(), 1);
}

#[test]
fn rejects_slow_receiver_touch_even_after_completed_pass() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut pass_calculator = PassCalculator::new();
    let mut one_timer = OneTimerCalculator::new();

    update_pass(
        &mut pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
        vec![touch(10, 1.0, passer, true)],
    );
    update_pass(
        &mut pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 400.0, 0.0)),
        vec![touch(20, 2.0, receiver.clone(), true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 400.0, 0.0)),
    );

    assert!(one_timer.player_stats().get(&receiver).is_none());
    assert!(one_timer.events().is_empty());
}

#[test]
fn rejects_fast_touch_not_aimed_toward_goal() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut pass_calculator = PassCalculator::new();
    let mut one_timer = OneTimerCalculator::new();

    update_pass(
        &mut pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
        vec![touch(10, 1.0, passer, true)],
    );
    update_pass(
        &mut pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(1600.0, 0.0, 0.0)),
        vec![touch(20, 2.0, receiver.clone(), true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(1600.0, 0.0, 0.0)),
    );

    assert!(one_timer.player_stats().get(&receiver).is_none());
    assert!(one_timer.events().is_empty());
}
