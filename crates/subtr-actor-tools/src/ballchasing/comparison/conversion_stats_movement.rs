use subtr_actor::{MovementStats, PowerslideStats};

use super::super::super::comparable_types::ComparableMovementStats;

pub(crate) fn comparable_movement_from_stats(
    movement: &MovementStats,
    powerslide: &PowerslideStats,
) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: Some(movement.average_speed() as f64),
        total_distance: Some(movement.total_distance as f64),
        time_supersonic_speed: Some(movement.time_supersonic_speed as f64),
        time_boost_speed: Some(movement.time_boost_speed as f64),
        time_slow_speed: Some(movement.time_slow_speed as f64),
        time_ground: Some(movement.time_on_ground as f64),
        time_low_air: Some(movement.time_low_air as f64),
        time_high_air: Some(movement.time_high_air as f64),
        time_powerslide: Some(powerslide.total_duration as f64),
        count_powerslide: Some(powerslide.press_count as f64),
        avg_powerslide_duration: Some(powerslide.average_duration() as f64),
        avg_speed_percentage: Some(movement.average_speed_pct() as f64),
        percent_slow_speed: Some(movement.slow_speed_pct() as f64),
        percent_boost_speed: Some(movement.boost_speed_pct() as f64),
        percent_supersonic_speed: Some(movement.supersonic_speed_pct() as f64),
        percent_ground: Some(movement.on_ground_pct() as f64),
        percent_low_air: Some(movement.low_air_pct() as f64),
        percent_high_air: Some(movement.high_air_pct() as f64),
    }
}
