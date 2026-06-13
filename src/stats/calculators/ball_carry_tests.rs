use super::*;
use crate::stats::calculators::ball_control_test_support::*;

#[test]
fn keeps_ground_carry_stats_separate_from_air_dribbles() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let x = i as f32 * 50.0;
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 120.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 20.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
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

    let stats = harness.calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.carry_count, 1);
    assert!(
        harness
            .calculator
            .player_air_dribble_stats()
            .get(&player_id)
            .is_none()
    );
    assert_eq!(
        harness.calculator.carry_events()[0].kind,
        BallCarryKind::Carry
    );
}
