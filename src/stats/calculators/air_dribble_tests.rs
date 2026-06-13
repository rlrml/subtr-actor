use super::*;
use crate::stats::calculators::ball_control_test_support::*;

#[test]
fn counts_air_dribble_when_airborne_player_carries_airborne_ball() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let x = i as f32 * 50.0;
        let touch_state = if matches!(i, 1 | 3 | 5) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    assert!(harness.calculator.player_stats().get(&player_id).is_none());
    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.ground_to_air_count, 1);
    assert_eq!(stats.wall_to_air_count, 0);
    assert_eq!(stats.total_touch_count, 3);
    assert!((stats.total_time - 1.0).abs() < f32::EPSILON);
    assert_eq!(
        harness.calculator.carry_events()[0].kind,
        BallCarryKind::AirDribble
    );
    assert_eq!(
        harness.calculator.carry_events()[0].air_dribble_origin,
        Some(AirDribbleOrigin::GroundToAir)
    );
}

#[test]
fn counts_takeoff_touch_plus_two_air_touches_as_air_dribble() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    harness.update(
        &frame(1, 0.2),
        &ball(
            glam::Vec3::new(50.0, 0.0, 520.0),
            glam::Vec3::new(250.0, 0.0, 0.0),
        ),
        &PlayerFrameState {
            players: vec![player(
                glam::Vec3::new(50.0, 0.0, 20.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            )],
        },
        &touch_state_with_touch(1, 0.2),
        &LivePlayState {
            is_live_play: true,
            ..LivePlayState::default()
        },
    );

    for i in 2..=6 {
        let x = i as f32 * 50.0;
        let touch_state = if matches!(i, 4 | 6) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.total_touch_count, 3);
    assert_eq!(harness.calculator.carry_events()[0].touch_count, 3);
    assert_eq!(harness.calculator.carry_events()[0].air_touch_count, 2);
}

#[test]
fn counts_air_dribble_that_finishes_inside_goal_mouth() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let y = 4700.0 + i as f32 * 100.0;
        let touch_state = if matches!(i, 1 | 3 | 5) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(120.0, y, 520.0),
                glam::Vec3::new(0.0, 250.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(80.0, y, 360.0),
                    glam::Vec3::new(0.0, 250.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.total_touch_count, 3);
}

#[test]
fn rejects_air_dribble_when_player_lands_between_touches() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=7 {
        let x = i as f32 * 50.0;
        let player_z = if matches!(i, 1 | 4) { 20.0 } else { 360.0 };
        let touch_state = if matches!(i, 1 | 3 | 7) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, player_z),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    assert!(
        harness
            .calculator
            .player_air_dribble_stats()
            .get(&player_id)
            .is_none()
    );
    assert!(harness.calculator.carry_events().is_empty());
}

#[test]
fn rejects_air_dribble_with_fewer_than_three_successive_touches() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let x = i as f32 * 50.0;
        let touch_state = if i == 3 {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    assert!(
        harness
            .calculator
            .player_air_dribble_stats()
            .get(&player_id)
            .is_none()
    );
    assert!(harness.calculator.carry_events().is_empty());
}

#[test]
fn records_air_dribble_touch_count() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=8 {
        let x = i as f32 * 50.0;
        let touch_state = if matches!(i, 1 | 4 | 7) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.total_touch_count, 3);
    assert_eq!(stats.max_touch_count, 3);
    assert_eq!(stats.average_touch_count(), 3.0);
    assert_eq!(harness.calculator.carry_events()[0].touch_count, 3);
}

#[test]
fn records_wall_to_air_dribble_origin() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let touch_state = if matches!(i, 1 | 3 | 5) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(3420.0, i as f32 * 50.0, 520.0),
                glam::Vec3::new(0.0, 250.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(3300.0, i as f32 * 50.0, 360.0),
                    glam::Vec3::new(0.0, 250.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.ground_to_air_count, 0);
    assert_eq!(stats.wall_to_air_count, 1);
    assert_eq!(
        harness.calculator.carry_events()[0].air_dribble_origin,
        Some(AirDribbleOrigin::WallToAir)
    );
}

#[test]
fn rejects_wall_control_as_air_dribble() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let y = i as f32 * 50.0;
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(3720.0, y, 520.0),
                glam::Vec3::new(0.0, 250.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(3650.0, y, 360.0),
                    glam::Vec3::new(0.0, 250.0, 0.0),
                )],
            },
            &touch_state(),
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    assert!(harness.calculator.player_stats().get(&player_id).is_none());
    assert!(
        harness
            .calculator
            .player_air_dribble_stats()
            .get(&player_id)
            .is_none()
    );
    assert!(harness.calculator.carry_events().is_empty());
}
