use serde_json::Value;

use super::super::super::comparable_types::ComparableMovementStats;
use super::json_number;

pub(crate) fn comparable_movement_from_json(stats: Option<&Value>) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: json_number(stats, "avg_speed"),
        total_distance: json_number(stats, "total_distance"),
        time_supersonic_speed: json_number(stats, "time_supersonic_speed"),
        time_boost_speed: json_number(stats, "time_boost_speed"),
        time_slow_speed: json_number(stats, "time_slow_speed"),
        time_ground: json_number(stats, "time_ground"),
        time_low_air: json_number(stats, "time_low_air"),
        time_high_air: json_number(stats, "time_high_air"),
        time_powerslide: json_number(stats, "time_powerslide"),
        count_powerslide: json_number(stats, "count_powerslide"),
        avg_powerslide_duration: json_number(stats, "avg_powerslide_duration"),
        avg_speed_percentage: json_number(stats, "avg_speed_percentage"),
        percent_slow_speed: json_number(stats, "percent_slow_speed"),
        percent_boost_speed: json_number(stats, "percent_boost_speed"),
        percent_supersonic_speed: json_number(stats, "percent_supersonic_speed"),
        percent_ground: json_number(stats, "percent_ground"),
        percent_low_air: json_number(stats, "percent_low_air"),
        percent_high_air: json_number(stats, "percent_high_air"),
    }
}
