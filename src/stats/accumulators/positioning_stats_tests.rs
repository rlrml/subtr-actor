use crate::stats::calculators::{BallProximityEvent, BallProximityState};

use super::*;

fn ball_proximity_event(
    player_id: u64,
    is_team_0: bool,
    duration: f32,
    state: BallProximityState,
) -> BallProximityEvent {
    BallProximityEvent {
        time: 0.0,
        frame: 0,
        end_time: duration,
        end_frame: 1,
        duration,
        player: boxcars::RemoteId::Steam(player_id),
        player_position: None,
        is_team_0,
        state,
    }
}

#[test]
fn positioning_accumulator_tracks_team_closest_to_ball_time() {
    let mut accumulator = PositioningStatsAccumulator::default();

    let events = [
        ball_proximity_event(
            1,
            true,
            0.1,
            BallProximityState {
                closest_to_ball_team: true,
                closest_to_ball_absolute: true,
                farthest_from_ball: false,
            },
        ),
        ball_proximity_event(
            2,
            false,
            0.2,
            BallProximityState {
                closest_to_ball_team: true,
                closest_to_ball_absolute: false,
                farthest_from_ball: false,
            },
        ),
        ball_proximity_event(
            3,
            true,
            0.3,
            BallProximityState {
                closest_to_ball_team: false,
                closest_to_ball_absolute: false,
                farthest_from_ball: true,
            },
        ),
    ];
    for event in &events {
        accumulator.apply_ball_proximity_event(event);
    }

    assert_eq!(accumulator.team_zero_stats().tracked_time, 0.1);
    assert_eq!(accumulator.team_zero_stats().time_closest_to_ball_team, 0.1);
    assert_eq!(
        accumulator.team_zero_stats().time_closest_to_ball_absolute,
        0.1
    );
    assert_eq!(
        accumulator.team_zero_stats().closest_to_ball_team_pct(),
        100.0
    );
    assert_eq!(accumulator.team_one_stats().tracked_time, 0.2);
    assert_eq!(accumulator.team_one_stats().time_closest_to_ball_team, 0.2);
    assert_eq!(
        accumulator.team_one_stats().time_closest_to_ball_absolute,
        0.0
    );
    assert_eq!(
        accumulator.team_one_stats().closest_to_ball_team_pct(),
        100.0
    );

    let player_one_stats = &accumulator.player_stats()[&boxcars::RemoteId::Steam(1)];
    assert_eq!(player_one_stats.time_closest_to_ball_team, 0.1);
    assert_eq!(player_one_stats.time_closest_to_ball_absolute, 0.1);
    let player_three_stats = &accumulator.player_stats()[&boxcars::RemoteId::Steam(3)];
    assert_eq!(player_three_stats.time_farthest_from_ball, 0.3);
}
