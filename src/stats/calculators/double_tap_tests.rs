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
            true,
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
fn followup_touch_rejects_trajectory_moving_away_from_goal_line() {
    let state = ball(
        glam::Vec3::new(0.0, 4200.0, 400.0),
        glam::Vec3::new(0.0, -1600.0, 0.0),
    );

    assert!(!DoubleTapCalculator::followup_touch_projects_on_goal_mouth(
        &state, true
    ));
}
