use subtr_actor::{DemoPlayerStats, DemoTeamStats, PositioningStats};

use super::super::super::comparable_types::{ComparableDemoStats, ComparablePositioningStats};

pub(crate) fn comparable_positioning_from_stats(
    stats: &PositioningStats,
) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: Some(stats.average_distance_to_ball() as f64),
        avg_distance_to_ball_possession: Some(
            stats.average_distance_to_ball_has_possession() as f64
        ),
        avg_distance_to_ball_no_possession: Some(
            stats.average_distance_to_ball_no_possession() as f64
        ),
        avg_distance_to_mates: Some(stats.average_distance_to_teammates() as f64),
        time_defensive_third: Some(stats.time_defensive_zone as f64),
        time_neutral_third: Some(stats.time_neutral_zone as f64),
        time_offensive_third: Some(stats.time_offensive_zone as f64),
        time_defensive_half: Some(stats.time_defensive_half as f64),
        time_offensive_half: Some(stats.time_offensive_half as f64),
        time_behind_ball: Some(stats.time_behind_ball as f64),
        time_infront_ball: Some(stats.time_in_front_of_ball as f64),
        time_most_back: Some(stats.time_most_back as f64),
        time_most_forward: Some(stats.time_most_forward as f64),
        time_closest_to_ball: Some(stats.time_closest_to_ball as f64),
        time_farthest_from_ball: Some(stats.time_farthest_from_ball as f64),
        percent_defensive_third: Some(stats.defensive_zone_pct() as f64),
        percent_neutral_third: Some(stats.neutral_zone_pct() as f64),
        percent_offensive_third: Some(stats.offensive_zone_pct() as f64),
        percent_defensive_half: Some(stats.defensive_half_pct() as f64),
        percent_offensive_half: Some(stats.offensive_half_pct() as f64),
        percent_behind_ball: Some(stats.behind_ball_pct() as f64),
        percent_infront_ball: Some(stats.in_front_of_ball_pct() as f64),
        percent_most_back: Some(stats.most_back_pct() as f64),
        percent_most_forward: Some(stats.most_forward_pct() as f64),
        percent_closest_to_ball: Some(stats.closest_to_ball_pct() as f64),
        percent_farthest_from_ball: Some(stats.farthest_from_ball_pct() as f64),
    }
}

pub(crate) fn comparable_demo_from_player(stats: &DemoPlayerStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: Some(stats.demos_taken as f64),
    }
}

pub(crate) fn comparable_demo_from_team(stats: &DemoTeamStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: None,
    }
}
