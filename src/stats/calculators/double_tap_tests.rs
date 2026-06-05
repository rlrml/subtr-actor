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

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
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
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

fn backboard_bounce(
    frame_number: usize,
    time: f32,
    player: PlayerId,
    is_team_0: bool,
) -> BackboardBounceState {
    backboard_bounce_with_position(frame_number, time, player, is_team_0, None)
}

fn backboard_bounce_with_position(
    frame_number: usize,
    time: f32,
    player: PlayerId,
    is_team_0: bool,
    player_position: Option<glam::Vec3>,
) -> BackboardBounceState {
    let event = BackboardBounceEvent {
        time,
        frame: frame_number,
        player,
        player_position: player_position.map(|position| position.to_array()),
        is_team_0,
    };
    BackboardBounceState {
        bounce_events: vec![event.clone()],
        last_bounce_event: Some(event),
    }
}

fn update(
    calculator: &mut DoubleTapCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    touch_events: Vec<TouchEvent>,
    backboard_bounce_state: BackboardBounceState,
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
            &LivePlayState::active_play(),
        )
        .unwrap();
}

#[test]
fn followup_touch_accepts_trajectory_projecting_inside_goal_mouth() {
    let state = ball(
        glam::Vec3::new(-352.88, 4740.9, 568.98),
        glam::Vec3::new(-690.64, 1833.67, -791.34),
    );

    assert!(DoubleTapCalculator::followup_touch_projects_on_goal_mouth(
        &state, true
    ));
}

#[test]
fn followup_touch_rejects_trajectory_projecting_wide_of_goal_mouth() {
    let state = ball(
        glam::Vec3::new(1200.0, 4200.0, 400.0),
        glam::Vec3::new(900.0, 1600.0, 0.0),
    );

    assert!(!DoubleTapCalculator::followup_touch_projects_on_goal_mouth(
        &state, true
    ));
}

#[test]
fn counts_matching_followup_even_with_same_frame_other_touch() {
    let shooter = PlayerId::Steam(1);
    let defender = PlayerId::Steam(2);
    let mut calculator = DoubleTapCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(
            glam::Vec3::new(0.0, 5000.0, 700.0),
            glam::Vec3::new(0.0, -1000.0, 0.0),
        ),
        Vec::new(),
        backboard_bounce(10, 1.0, shooter.clone(), true),
    );
    update(
        &mut calculator,
        frame(20, 1.2),
        ball(
            glam::Vec3::new(0.0, 4500.0, 400.0),
            glam::Vec3::new(0.0, 1600.0, 0.0),
        ),
        vec![
            touch(20, 1.2, shooter.clone(), true),
            touch(20, 1.2, defender, false),
        ],
        BackboardBounceState::default(),
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].player, shooter);
}

#[test]
fn aggregate_followup_uses_latest_matching_touch_time() {
    let shooter = PlayerId::Steam(1);
    let mut calculator = DoubleTapCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(
            glam::Vec3::new(0.0, 5000.0, 700.0),
            glam::Vec3::new(0.0, -1000.0, 0.0),
        ),
        Vec::new(),
        backboard_bounce(10, 1.0, shooter.clone(), true),
    );
    update(
        &mut calculator,
        frame(30, 1.4),
        ball(
            glam::Vec3::new(0.0, 4500.0, 400.0),
            glam::Vec3::new(0.0, 1600.0, 0.0),
        ),
        vec![
            touch(20, 1.2, shooter.clone(), true),
            touch(25, 1.3, shooter.clone(), true),
        ],
        BackboardBounceState::default(),
    );

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].player, shooter);
    assert_eq!(calculator.events()[0].frame, 25);
    assert_eq!(calculator.events()[0].time, 1.3);
}

#[test]
fn rejects_backboard_touch_from_grounded_player() {
    let shooter = PlayerId::Steam(1);
    let mut calculator = DoubleTapCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball(
            glam::Vec3::new(0.0, 5000.0, 700.0),
            glam::Vec3::new(0.0, -1000.0, 0.0),
        ),
        Vec::new(),
        backboard_bounce_with_position(
            10,
            1.0,
            shooter.clone(),
            true,
            Some(glam::Vec3::new(0.0, 0.0, PLAYER_GROUND_Z_THRESHOLD)),
        ),
    );
    update(
        &mut calculator,
        frame(20, 1.2),
        ball(
            glam::Vec3::new(0.0, 4500.0, 400.0),
            glam::Vec3::new(0.0, 1600.0, 0.0),
        ),
        vec![touch(20, 1.2, shooter, true)],
        BackboardBounceState::default(),
    );

    assert!(calculator.events().is_empty());
}

#[test]
fn followup_touch_rejects_trajectory_moving_away_from_goal_line() {
    let state = ball(
        glam::Vec3::new(0.0, 4200.0, 400.0),
        glam::Vec3::new(0.0, -1600.0, 0.0),
    );

    assert!(!DoubleTapCalculator::followup_touch_projects_on_goal_mouth(
        &state, true
    ));
}
