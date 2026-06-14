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
        touch_id: None,
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: Some(player_id),
        player_position: None,
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
            &LivePlayState::active_play(),
        )
        .unwrap();
}

fn update_pass_with_backboard(
    calculator: &mut PassCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
    backboard_bounce_state: &BackboardBounceState,
) {
    calculator
        .update(
            &frame,
            &ball,
            &TouchState {
                touch_events,
                ..TouchState::default()
            },
            backboard_bounce_state,
            &FiftyFiftyState::default(),
            &LivePlayState::active_play(),
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
        .update(
            &frame,
            &ball,
            pass_calculator,
            &LivePlayState::active_play(),
        )
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
fn rejects_backboard_pass_double_tap() {
    let passer = PlayerId::Steam(1);
    let receiver = PlayerId::Steam(2);
    let mut pass_calculator = PassCalculator::new();
    let mut one_timer = OneTimerCalculator::new();

    update_pass(
        &mut pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
        vec![touch(10, 1.0, passer.clone(), true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(10, 1.0),
        ball(0.0, glam::Vec3::ZERO),
    );

    // The passer knocks the ball off the backboard before the teammate finishes
    // it: this is a double tap / backboard play, not a one-timer.
    let backboard_bounce_state = BackboardBounceState {
        last_bounce_event: Some(BackboardBounceEvent {
            time: 1.5,
            frame: 15,
            player: passer,
            player_position: None,
            is_team_0: true,
        }),
        ..BackboardBounceState::default()
    };
    update_pass_with_backboard(
        &mut pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 1600.0, 0.0)),
        vec![touch(20, 2.0, receiver.clone(), true)],
        &backboard_bounce_state,
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(20, 2.0),
        ball(800.0, glam::Vec3::new(0.0, 1600.0, 0.0)),
    );

    // The underlying pass is detected, but classified as a backboard pass...
    assert_eq!(pass_calculator.events().len(), 1);
    assert_eq!(pass_calculator.events()[0].pass_kind, PassKind::Backboard);
    // ...so it must not be counted as a one-timer.
    assert!(one_timer.player_stats().get(&receiver).is_none());
    assert!(one_timer.events().is_empty());
}

#[test]
fn rejects_fast_goal_aligned_touch_that_sails_wide_of_net() {
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
    // Fast and pointed at the goal's general direction (passes the alignment
    // cosine), but the sideways component carries it well wide of the posts:
    // off net, so it is not a one-timer.
    let velocity = glam::Vec3::new(600.0, 1500.0, 0.0);
    update_pass(
        &mut pass_calculator,
        frame(20, 2.0),
        ball(800.0, velocity),
        vec![touch(20, 2.0, receiver.clone(), true)],
    );
    update_one_timer(
        &mut one_timer,
        &pass_calculator,
        frame(20, 2.0),
        ball(800.0, velocity),
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
