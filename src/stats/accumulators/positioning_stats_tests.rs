use crate::stats::calculators::{
    PositioningEvent, PositioningPossessionState, PositioningTeammateRoleState,
};

use super::*;

fn positioning_event(
    player_id: u64,
    is_team_0: bool,
    duration: f32,
    tracked: bool,
    closest_to_ball: bool,
) -> PositioningEvent {
    PositioningEvent {
        time: 0.0,
        frame: 0,
        end_time: duration,
        end_frame: 1,
        duration,
        player: boxcars::RemoteId::Steam(player_id),
        player_position: None,
        is_team_0,
        active: true,
        tracked,
        distance_to_teammates: None,
        distance_to_ball: None,
        possession_state: PositioningPossessionState::Neutral,
        demolished: false,
        no_teammates: false,
        teammate_role: PositioningTeammateRoleState::Unknown,
        defensive_zone_fraction: 0.0,
        neutral_zone_fraction: 0.0,
        offensive_zone_fraction: 0.0,
        defensive_half_fraction: 0.0,
        offensive_half_fraction: 0.0,
        closest_to_ball,
        closest_to_ball_team: closest_to_ball,
        closest_to_ball_absolute: closest_to_ball && player_id == 1,
        farthest_from_ball: false,
        behind_ball_fraction: 0.0,
        level_with_ball_fraction: 0.0,
        in_front_of_ball_fraction: 0.0,
    }
}

#[test]
fn positioning_accumulator_tracks_team_closest_to_ball_time() {
    let mut accumulator = PositioningStatsAccumulator::default();

    let events = [
        positioning_event(1, true, 0.1, true, true),
        positioning_event(2, false, 0.2, true, true),
        positioning_event(3, true, 0.3, true, false),
        positioning_event(4, false, 0.4, false, true),
    ];
    accumulator.apply_events(events.iter());

    assert_eq!(accumulator.team_zero_stats().tracked_time, 0.1);
    assert_eq!(accumulator.team_zero_stats().time_closest_to_ball, 0.1);
    assert_eq!(accumulator.team_zero_stats().time_closest_to_ball_team, 0.1);
    assert_eq!(
        accumulator.team_zero_stats().time_closest_to_ball_absolute,
        0.1
    );
    assert_eq!(accumulator.team_zero_stats().closest_to_ball_pct(), 100.0);
    assert_eq!(accumulator.team_one_stats().tracked_time, 0.2);
    assert_eq!(accumulator.team_one_stats().time_closest_to_ball, 0.2);
    assert_eq!(accumulator.team_one_stats().time_closest_to_ball_team, 0.2);
    assert_eq!(
        accumulator.team_one_stats().time_closest_to_ball_absolute,
        0.0
    );
    assert_eq!(accumulator.team_one_stats().closest_to_ball_pct(), 100.0);
}
