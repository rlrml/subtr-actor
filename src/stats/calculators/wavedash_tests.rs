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

fn player(id: u64, z: f32, horizontal_speed: f32, dodge_active: bool) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0: true,
        rigid_body: Some(rigid_body(
            glam::Vec3::new(0.0, 0.0, z),
            glam::Vec3::new(horizontal_speed, 0.0, 0.0),
        )),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
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
        dt: 0.05,
        seconds_remaining: None,
    }
}

fn players(player: PlayerSample) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player],
    }
}

#[test]
fn counts_air_dodge_that_lands_quickly() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WavedashCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, 55.0, 700.0, true)),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 1.14),
            &players(player(1, 17.0, 1300.0, true)),
            true,
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert!(stats.last_quality.unwrap() >= WAVEDASH_MIN_CONFIDENCE);
    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].frame, 2);
    assert!((calculator.events()[0].horizontal_speed_gain - 600.0).abs() <= f32::EPSILON);
}

#[test]
fn rejects_late_landing_after_air_dodge() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WavedashCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, 80.0, 700.0, true)),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 1.42),
            &players(player(1, 17.0, 1500.0, true)),
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_grounded_dodge_start() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WavedashCalculator::new();

    calculator
        .update(
            &frame(1, 1.00),
            &players(player(1, 17.0, 700.0, true)),
            true,
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 1.08),
            &players(player(1, 17.0, 1500.0, true)),
            true,
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}
