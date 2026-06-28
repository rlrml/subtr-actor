use super::*;

fn frame(frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: None,
    }
}

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

fn player(player_id: PlayerId, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: CarHitbox::octane(),
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
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

fn players(samples: Vec<PlayerSample>) -> PlayerFrameState {
    PlayerFrameState { players: samples }
}

fn touch(player: PlayerId, is_team_0: bool, gap: f32) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time: 0.1,
        frame: 1,
        team_is_team_0: is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(gap),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

#[test]
fn backboard_bounce_uses_primary_touch_not_last_contested_candidate() {
    let primary_player = boxcars::RemoteId::Steam(1);
    let secondary_player = boxcars::RemoteId::Steam(2);
    let primary_touch = touch(primary_player.clone(), true, 0.0);
    let secondary_touch = touch(secondary_player, false, 3.0);
    let touch_state = TouchState {
        touch_events: vec![primary_touch.clone(), secondary_touch],
        last_touch: Some(primary_touch),
        last_touch_player: Some(primary_player.clone()),
        last_touch_team_is_team_0: Some(true),
    };
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            glam::Vec3::new(0.0, 4700.0, 650.0),
            glam::Vec3::new(0.0, 500.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &touch_state,
        &LivePlayState::active_play(),
    );
    let state = calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(0.0, 4800.0, 650.0),
            glam::Vec3::new(0.0, -300.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState::default(),
        &LivePlayState::active_play(),
    );

    let [bounce] = state.bounce_events.as_slice() else {
        panic!("expected exactly one backboard bounce");
    };
    assert_eq!(bounce.player, primary_player);
    assert!(bounce.is_team_0);
}

#[test]
fn wide_back_wall_rebound_counts_as_backboard_bounce() {
    let shooter = boxcars::RemoteId::Steam(1);
    let touch_state = TouchState {
        touch_events: vec![touch(shooter.clone(), true, 0.0)],
        last_touch: None,
        last_touch_player: None,
        last_touch_team_is_team_0: None,
    };
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            // Mirrors wide back-wall reads that are visually double taps even
            // though they are outside the old central-backboard gate.
            glam::Vec3::new(2746.0, 4840.0, 1400.0),
            glam::Vec3::new(0.0, 500.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &touch_state,
        &LivePlayState::active_play(),
    );
    let state = calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(2746.0, 4800.0, 1400.0),
            glam::Vec3::new(0.0, -300.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState::default(),
        &LivePlayState::active_play(),
    );

    let [bounce] = state.bounce_events.as_slice() else {
        panic!("expected exactly one wide back-wall bounce");
    };
    assert_eq!(bounce.player, shooter);
    assert!(bounce.is_team_0);
}

#[test]
fn backboard_bounce_can_emit_on_simultaneous_touch_frame() {
    let shooter = boxcars::RemoteId::Steam(1);
    let initial_touch = touch(shooter.clone(), true, 0.0);
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            glam::Vec3::new(0.0, 4700.0, 650.0),
            glam::Vec3::new(0.0, 1000.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState {
            touch_events: vec![initial_touch.clone()],
            last_touch: Some(initial_touch),
            last_touch_player: Some(shooter.clone()),
            last_touch_team_is_team_0: Some(true),
        },
        &LivePlayState::active_play(),
    );

    let state = calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(0.0, 5000.0, 650.0),
            glam::Vec3::new(0.0, -1000.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState {
            touch_events: vec![TouchEvent {
                time: 0.2,
                frame: 2,
                ..touch(shooter.clone(), true, 0.0)
            }],
            ..TouchState::default()
        },
        &LivePlayState::active_play(),
    );

    let [bounce] = state.bounce_events.as_slice() else {
        panic!("expected exactly one simultaneous-frame backboard bounce");
    };
    assert_eq!(bounce.player, shooter);
    assert!(bounce.is_team_0);
    assert_eq!(bounce.frame, 2);
}

#[test]
fn simultaneous_backboard_touch_does_not_require_rebound_velocity_sample() {
    let shooter = boxcars::RemoteId::Steam(1);
    let initial_touch = touch(shooter.clone(), true, 0.0);
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            glam::Vec3::new(0.0, 4700.0, 650.0),
            glam::Vec3::new(0.0, 1000.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState {
            touch_events: vec![initial_touch.clone()],
            last_touch: Some(initial_touch),
            last_touch_player: Some(shooter.clone()),
            last_touch_team_is_team_0: Some(true),
        },
        &LivePlayState::active_play(),
    );

    let state = calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(0.0, 5030.0, 650.0),
            glam::Vec3::new(0.0, 1000.0, 0.0),
        ),
        &PlayerFrameState::default(),
        &TouchState {
            touch_events: vec![TouchEvent {
                time: 0.2,
                frame: 2,
                ..touch(shooter.clone(), true, 0.0)
            }],
            ..TouchState::default()
        },
        &LivePlayState::active_play(),
    );

    let [bounce] = state.bounce_events.as_slice() else {
        panic!("expected simultaneous touch to count as the backboard contact");
    };
    assert_eq!(bounce.player, shooter);
    assert!(bounce.is_team_0);
    assert_eq!(bounce.frame, 2);
}

#[test]
fn surface_contact_between_touch_and_bounce_drops_attribution() {
    let shooter = boxcars::RemoteId::Steam(1);
    let touch_state = TouchState {
        touch_events: vec![touch(shooter.clone(), true, 0.0)],
        last_touch: None,
        last_touch_player: None,
        last_touch_team_is_team_0: None,
    };
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            glam::Vec3::new(0.0, 4700.0, 650.0),
            glam::Vec3::new(0.0, 500.0, 0.0),
        ),
        &players(vec![player(
            shooter.clone(),
            true,
            glam::Vec3::new(0.0, 3000.0, PLAYER_GROUND_Z_THRESHOLD + 200.0),
        )]),
        &touch_state,
        &LivePlayState::active_play(),
    );
    calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(0.0, 4750.0, 650.0),
            glam::Vec3::new(0.0, 500.0, 0.0),
        ),
        &players(vec![player(
            shooter.clone(),
            true,
            glam::Vec3::new(0.0, 3000.0, PLAYER_GROUND_Z_THRESHOLD),
        )]),
        &TouchState::default(),
        &LivePlayState::active_play(),
    );
    let state = calculator.update(
        &frame(3),
        &ball(
            glam::Vec3::new(0.0, 4800.0, 650.0),
            glam::Vec3::new(0.0, -300.0, 0.0),
        ),
        &players(vec![player(
            shooter,
            true,
            glam::Vec3::new(0.0, 3000.0, PLAYER_GROUND_Z_THRESHOLD + 200.0),
        )]),
        &TouchState::default(),
        &LivePlayState::active_play(),
    );

    assert!(state.bounce_events.is_empty());
}
