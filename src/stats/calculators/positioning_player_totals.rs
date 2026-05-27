use super::*;

pub(crate) fn record_live_player_totals(
    stats: &mut PositioningStats,
    delta: &mut PositioningEvent,
    dt: f32,
    distance_to_ball: f32,
) {
    stats.active_game_time += dt;
    stats.tracked_time += dt;
    stats.sum_distance_to_ball += distance_to_ball * dt;
    delta.active_game_time += dt;
    delta.tracked_time += dt;
    delta.sum_distance_to_ball += distance_to_ball * dt;
}

pub(crate) fn record_possession_distance(
    stats: &mut PositioningStats,
    delta: &mut PositioningEvent,
    dt: f32,
    distance_to_ball: f32,
    has_possession: bool,
    possession_known: bool,
) {
    if has_possession {
        stats.time_has_possession += dt;
        stats.sum_distance_to_ball_has_possession += distance_to_ball * dt;
        delta.time_has_possession += dt;
        delta.sum_distance_to_ball_has_possession += distance_to_ball * dt;
    } else if possession_known {
        stats.time_no_possession += dt;
        stats.sum_distance_to_ball_no_possession += distance_to_ball * dt;
        delta.time_no_possession += dt;
        delta.sum_distance_to_ball_no_possession += distance_to_ball * dt;
    }
}

pub(crate) fn record_field_positioning(
    stats: &mut PositioningStats,
    delta: &mut PositioningEvent,
    dt: f32,
    is_team_0: bool,
    previous_position: glam::Vec3,
    position: glam::Vec3,
) {
    let previous_y = normalized_y(is_team_0, previous_position);
    let y = normalized_y(is_team_0, position);
    let defensive = interval_fraction_below_threshold(previous_y, y, -FIELD_ZONE_BOUNDARY_Y);
    let offensive = interval_fraction_above_threshold(previous_y, y, FIELD_ZONE_BOUNDARY_Y);
    let neutral = interval_fraction_in_scalar_range(
        previous_y,
        y,
        -FIELD_ZONE_BOUNDARY_Y,
        FIELD_ZONE_BOUNDARY_Y,
    );
    stats.time_defensive_zone += dt * defensive;
    stats.time_neutral_zone += dt * neutral;
    stats.time_offensive_zone += dt * offensive;
    delta.time_defensive_zone += dt * defensive;
    delta.time_neutral_zone += dt * neutral;
    delta.time_offensive_zone += dt * offensive;

    let defensive_half = interval_fraction_below_threshold(previous_y, y, 0.0);
    stats.time_defensive_half += dt * defensive_half;
    stats.time_offensive_half += dt * (1.0 - defensive_half);
    delta.time_defensive_half += dt * defensive_half;
    delta.time_offensive_half += dt * (1.0 - defensive_half);
}
