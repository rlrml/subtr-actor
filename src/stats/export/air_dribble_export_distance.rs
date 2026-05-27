use crate::{AirDribbleStats, ExportedStat, StatUnit};

pub(super) fn visit_air_dribble_distance_fields(
    stats: &AirDribbleStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "air_dribble",
        "total_straight_line_distance",
        StatUnit::UnrealUnits,
        stats.total_straight_line_distance,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_straight_line_distance",
        StatUnit::UnrealUnits,
        stats.average_straight_line_distance(),
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "furthest_straight_line_distance",
        StatUnit::UnrealUnits,
        stats.furthest_distance,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "total_path_distance",
        StatUnit::UnrealUnits,
        stats.total_path_distance,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_path_distance",
        StatUnit::UnrealUnits,
        stats.average_path_distance(),
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_speed",
        StatUnit::UnrealUnitsPerSecond,
        stats.average_speed(),
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "fastest_avg_speed",
        StatUnit::UnrealUnitsPerSecond,
        stats.fastest_speed,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_horizontal_gap",
        StatUnit::UnrealUnits,
        stats.average_horizontal_gap(),
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_vertical_gap",
        StatUnit::UnrealUnits,
        stats.average_vertical_gap(),
    ));
}
