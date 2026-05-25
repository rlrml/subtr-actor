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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.2,
        seconds_remaining: None,
    }
}

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn player_at(position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn players(position: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_at(position)],
    }
}

fn controlled_touch_state() -> TouchState {
    TouchState {
        last_touch_player: Some(boxcars::RemoteId::Steam(1)),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    }
}

fn touch_state_with_touch(frame: usize, time: f32) -> TouchState {
    let player = boxcars::RemoteId::Steam(1);
    TouchState {
        touch_events: vec![TouchEvent {
            time,
            frame,
            team_is_team_0: true,
            player: Some(player.clone()),
            closest_approach_distance: Some(0.0),
        }],
        last_touch: Some(TouchEvent {
            time,
            frame,
            team_is_team_0: true,
            player: Some(player.clone()),
            closest_approach_distance: Some(0.0),
        }),
        last_touch_player: Some(player),
        last_touch_team_is_team_0: Some(true),
    }
}

#[test]
fn records_controlled_wall_aerial_play_after_wall_carry_setup() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3520.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3650.0, 0.0, 250.0)),
            &controlled_touch_state(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 340.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3650.0, 0.0, 270.0)),
            &controlled_touch_state(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3400.0, 0.0, 390.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 320.0)),
            &controlled_touch_state(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 430.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(4, 0.8),
            true,
        )
        .unwrap();

    let event = calculator.events().first().expect("wall aerial event");
    assert_eq!(event.player, player.clone());
    assert_eq!(event.wall, WallAerialWall::Side);
    assert!(event.setup_duration >= WALL_AERIAL_MIN_CONTROL_DURATION);

    let stats = calculator.player_stats().get(&player).unwrap();
    assert_eq!(stats.count, 1);
}

#[test]
fn rejects_wall_aerial_play_without_wall_control_setup() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3650.0, 0.0, 260.0)),
            &TouchState::default(),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball(
                glam::Vec3::new(3350.0, 0.0, 420.0),
                glam::Vec3::new(0.0, 700.0, 0.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(2, 0.2),
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}
