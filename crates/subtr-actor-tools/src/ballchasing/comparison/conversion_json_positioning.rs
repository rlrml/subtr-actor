use serde_json::Value;

use super::super::super::comparable_types::ComparablePositioningStats;
use super::json_number;

pub(crate) fn comparable_positioning_from_json(
    stats: Option<&Value>,
) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: json_number(stats, "avg_distance_to_ball"),
        avg_distance_to_ball_possession: json_number(stats, "avg_distance_to_ball_possession"),
        avg_distance_to_ball_no_possession: json_number(
            stats,
            "avg_distance_to_ball_no_possession",
        ),
        avg_distance_to_mates: json_number(stats, "avg_distance_to_mates"),
        time_defensive_third: json_number(stats, "time_defensive_third"),
        time_neutral_third: json_number(stats, "time_neutral_third"),
        time_offensive_third: json_number(stats, "time_offensive_third"),
        time_defensive_half: json_number(stats, "time_defensive_half"),
        time_offensive_half: json_number(stats, "time_offensive_half"),
        time_behind_ball: json_number(stats, "time_behind_ball"),
        time_infront_ball: json_number(stats, "time_infront_ball"),
        time_most_back: json_number(stats, "time_most_back"),
        time_most_forward: json_number(stats, "time_most_forward"),
        time_closest_to_ball: json_number(stats, "time_closest_to_ball"),
        time_farthest_from_ball: json_number(stats, "time_farthest_from_ball"),
        percent_defensive_third: json_number(stats, "percent_defensive_third"),
        percent_neutral_third: json_number(stats, "percent_neutral_third"),
        percent_offensive_third: json_number(stats, "percent_offensive_third"),
        percent_defensive_half: json_number(stats, "percent_defensive_half"),
        percent_offensive_half: json_number(stats, "percent_offensive_half"),
        percent_behind_ball: json_number(stats, "percent_behind_ball"),
        percent_infront_ball: json_number(stats, "percent_infront_ball"),
        percent_most_back: json_number(stats, "percent_most_back"),
        percent_most_forward: json_number(stats, "percent_most_forward"),
        percent_closest_to_ball: json_number(stats, "percent_closest_to_ball"),
        percent_farthest_from_ball: json_number(stats, "percent_farthest_from_ball"),
    }
}
