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

fn player_at(id: u64, is_team_0: bool, position: glam::Vec3, velocity: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), glam::Vec3::ZERO),
    })
}

fn touch_by(id: u64, is_team_0: bool, frame_number: usize, time: f32) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: Some(boxcars::RemoteId::Steam(id)),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn touch_state_with(touches: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events: touches,
        ..TouchState::default()
    }
}

/// Feeds `frames` of live play where the loser (id 1, team 0) converges on the
/// ball from `start_x` at `speed` uu/s, and the winner (id 2, team 1) sits near
/// the ball. Returns the frame number/time reached.
fn run_convergence(
    calculator: &mut BeatenToBallCalculator,
    frames: usize,
    start_x: f32,
    speed: f32,
) -> (usize, f32) {
    let ball = ball();
    for step in 1..=frames {
        let time = step as f32 * 0.1;
        let x = start_x + speed * (time - 0.1);
        calculator
            .update(
                &frame(step, time),
                &ball,
                &PlayerFrameState {
                    players: vec![
                        player_at(
                            1,
                            true,
                            glam::Vec3::new(x, 0.0, 17.0),
                            glam::Vec3::new(speed, 0.0, 0.0),
                        ),
                        player_at(
                            2,
                            false,
                            glam::Vec3::new(150.0, 0.0, 17.0),
                            glam::Vec3::new(-100.0, 0.0, 0.0),
                        ),
                    ],
                },
                &TouchState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }
    (frames, frames as f32 * 0.1)
}

fn touch_frame_update(
    calculator: &mut BeatenToBallCalculator,
    frame_number: usize,
    time: f32,
    loser_x: f32,
    loser_speed: f32,
) {
    calculator
        .update(
            &frame(frame_number, time),
            &ball(),
            &PlayerFrameState {
                players: vec![
                    player_at(
                        1,
                        true,
                        glam::Vec3::new(loser_x, 0.0, 17.0),
                        glam::Vec3::new(loser_speed, 0.0, 0.0),
                    ),
                    player_at(
                        2,
                        false,
                        glam::Vec3::new(80.0, 0.0, 17.0),
                        glam::Vec3::new(-100.0, 0.0, 0.0),
                    ),
                ],
            },
            &touch_state_with(vec![touch_by(2, false, frame_number, time)]),
            &LivePlayState::active_play(),
        )
        .unwrap();
}

#[test]
fn converging_opponent_beaten_narrowly_emits_event() {
    let mut calculator = BeatenToBallCalculator::new();
    // Frames 1..=9: loser closes from -1200 toward the ball at 900 uu/s.
    run_convergence(&mut calculator, 9, -1200.0, 900.0);
    // Frame 10: winner (team 1) touches while the loser is ~390 uu away.
    touch_frame_update(&mut calculator, 10, 1.0, -390.0, 900.0);

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.player, boxcars::RemoteId::Steam(1));
    assert_eq!(event.winner, boxcars::RemoteId::Steam(2));
    assert!(event.is_team_0);
    assert_eq!(event.frame, 10);
    assert!(event.approach_speed > 800.0);
    assert!(event.velocity_alignment > 0.9);
    assert!(
        event.margin_seconds > 0.0 && event.margin_seconds < 0.75,
        "margin should be a plausible narrow loss, got {}",
        event.margin_seconds
    );
    assert!(event.distance_at_touch < 500.0);
    assert!(!event.dodge_active);
    assert!(!event.aerial);
}

#[test]
fn retreating_opponent_is_not_beaten_to_ball() {
    let mut calculator = BeatenToBallCalculator::new();
    let ball = ball();
    // Loser drives away from the ball the whole window.
    for step in 1..=9usize {
        let time = step as f32 * 0.1;
        let x = -400.0 - 900.0 * (time - 0.1);
        calculator
            .update(
                &frame(step, time),
                &ball,
                &PlayerFrameState {
                    players: vec![
                        player_at(
                            1,
                            true,
                            glam::Vec3::new(x, 0.0, 17.0),
                            glam::Vec3::new(-900.0, 0.0, 0.0),
                        ),
                        player_at(
                            2,
                            false,
                            glam::Vec3::new(150.0, 0.0, 17.0),
                            glam::Vec3::new(-100.0, 0.0, 0.0),
                        ),
                    ],
                },
                &TouchState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }
    calculator
        .update(
            &frame(10, 1.0),
            &ball,
            &PlayerFrameState {
                players: vec![
                    player_at(
                        1,
                        true,
                        glam::Vec3::new(-1210.0, 0.0, 17.0),
                        glam::Vec3::new(-900.0, 0.0, 0.0),
                    ),
                    player_at(
                        2,
                        false,
                        glam::Vec3::new(80.0, 0.0, 17.0),
                        glam::Vec3::new(-100.0, 0.0, 0.0),
                    ),
                ],
            },
            &touch_state_with(vec![touch_by(2, false, 10, 1.0)]),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn distant_opponent_is_not_beaten_to_ball() {
    let mut calculator = BeatenToBallCalculator::new();
    // Converging hard, but still ~2200 uu away at the touch.
    run_convergence(&mut calculator, 9, -3000.0, 900.0);
    touch_frame_update(&mut calculator, 10, 1.0, -2190.0, 900.0);

    assert!(calculator.events().is_empty());
}

#[test]
fn opponent_who_touched_recently_is_not_beaten_to_ball() {
    let mut calculator = BeatenToBallCalculator::new();
    let ball = ball();
    run_convergence(&mut calculator, 4, -1200.0, 900.0);
    // Frame 5: the would-be loser gets a touch of their own.
    calculator
        .update(
            &frame(5, 0.5),
            &ball,
            &PlayerFrameState {
                players: vec![
                    player_at(
                        1,
                        true,
                        glam::Vec3::new(-840.0, 0.0, 17.0),
                        glam::Vec3::new(900.0, 0.0, 0.0),
                    ),
                    player_at(
                        2,
                        false,
                        glam::Vec3::new(150.0, 0.0, 17.0),
                        glam::Vec3::new(-100.0, 0.0, 0.0),
                    ),
                ],
            },
            &touch_state_with(vec![touch_by(1, true, 5, 0.5)]),
            &LivePlayState::active_play(),
        )
        .unwrap();
    for step in 6..=9usize {
        let time = step as f32 * 0.1;
        let x = -1200.0 + 900.0 * (time - 0.1);
        calculator
            .update(
                &frame(step, time),
                &ball,
                &PlayerFrameState {
                    players: vec![
                        player_at(
                            1,
                            true,
                            glam::Vec3::new(x, 0.0, 17.0),
                            glam::Vec3::new(900.0, 0.0, 0.0),
                        ),
                        player_at(
                            2,
                            false,
                            glam::Vec3::new(150.0, 0.0, 17.0),
                            glam::Vec3::new(-100.0, 0.0, 0.0),
                        ),
                    ],
                },
                &TouchState::default(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }
    // Winner touches at frame 10, but the loser touched 0.5s ago.
    touch_frame_update(&mut calculator, 10, 1.0, -390.0, 900.0);

    assert!(calculator.events().is_empty());
}

#[test]
fn rate_limit_emits_one_event_for_quick_successive_touches() {
    let mut calculator = BeatenToBallCalculator::new();
    run_convergence(&mut calculator, 9, -1200.0, 900.0);
    touch_frame_update(&mut calculator, 10, 1.0, -390.0, 900.0);
    // A second winning touch 0.1s later would re-qualify the same loser, but
    // the per-loser cooldown suppresses it.
    touch_frame_update(&mut calculator, 11, 1.1, -300.0, 900.0);

    assert_eq!(calculator.events().len(), 1);
}
