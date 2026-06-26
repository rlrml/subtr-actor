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

fn player_at(position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
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

fn players(position: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_at(position)],
    }
}

fn shot_event(frame: usize, time: f32, ball_body: &boxcars::RigidBody) -> PlayerStatEvent {
    let player = boxcars::RemoteId::Steam(1);
    let player_body = rigid_body(glam::Vec3::new(3350.0, 0.0, 340.0), glam::Vec3::ZERO);
    PlayerStatEvent {
        time,
        frame,
        player: player.clone(),
        player_position: None,
        is_team_0: true,
        kind: PlayerStatEventKind::Shot,
        shot: Some(ShotEventMetadata::from_rigid_bodies(
            true,
            ball_body,
            Some(&player_body),
        )),
    }
}

#[test]
fn records_wall_aerial_shot_without_requiring_prior_control() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialShotCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &players(glam::Vec3::new(3650.0, 0.0, 260.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &players(glam::Vec3::new(3350.0, 0.0, 330.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let shot_ball = rigid_body(
        glam::Vec3::new(3300.0, 0.0, 410.0),
        glam::Vec3::new(-300.0, 1400.0, 20.0),
    );
    calculator
        .update(
            &frame(3, 0.4),
            &players(glam::Vec3::new(3350.0, 0.0, 340.0)),
            &FrameEventsState {
                player_stat_events: vec![shot_event(3, 0.4, &shot_ball)],
                ..FrameEventsState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = calculator.events().first().expect("wall aerial shot event");
    assert_eq!(event.player, player.clone());
    assert_eq!(event.wall, WallAerialWall::Side);

    let stats = calculator.player_stats().get(&player).unwrap();
    assert_eq!(stats.count, 1);
}

#[test]
fn rejects_wall_aerial_shot_after_returning_low_from_wall() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialShotCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &players(glam::Vec3::new(3650.0, 0.0, 260.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &players(glam::Vec3::new(2500.0, 0.0, 80.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let shot_ball = rigid_body(
        glam::Vec3::new(3300.0, 0.0, 410.0),
        glam::Vec3::new(-300.0, 1400.0, 20.0),
    );
    calculator
        .update(
            &frame(3, 0.4),
            &players(glam::Vec3::new(3350.0, 0.0, 340.0)),
            &FrameEventsState {
                player_stat_events: vec![shot_event(3, 0.4, &shot_ball)],
                ..FrameEventsState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_wall_aerial_shot_from_stale_wall_contact() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialShotCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &players(glam::Vec3::new(3650.0, 0.0, 260.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 2.6),
            &players(glam::Vec3::new(3350.0, 0.0, 330.0)),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let shot_ball = rigid_body(
        glam::Vec3::new(3300.0, 0.0, 410.0),
        glam::Vec3::new(-300.0, 1400.0, 20.0),
    );
    calculator
        .update(
            &frame(3, 2.8),
            &players(glam::Vec3::new(3350.0, 0.0, 340.0)),
            &FrameEventsState {
                player_stat_events: vec![shot_event(3, 2.8, &shot_ball)],
                ..FrameEventsState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_wall_aerial_shot_without_wall_takeoff() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialShotCalculator::new();
    let shot_ball = rigid_body(
        glam::Vec3::new(3300.0, 0.0, 410.0),
        glam::Vec3::new(-300.0, 1400.0, 20.0),
    );

    calculator
        .update(
            &frame(1, 0.0),
            &players(glam::Vec3::new(3350.0, 0.0, 340.0)),
            &FrameEventsState {
                player_stat_events: vec![shot_event(1, 0.0, &shot_ball)],
                ..FrameEventsState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}
