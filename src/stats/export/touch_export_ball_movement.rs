use crate::{ExportedStat, StatUnit, TouchStats};

pub(super) fn visit_touch_ball_movement_fields(
    stats: &TouchStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    if let Some(value) = stats.last_ball_speed_change {
        visitor(ExportedStat::float(
            "touch",
            "last_ball_speed_change",
            StatUnit::UnrealUnitsPerSecond,
            value,
        ));
    }
    visitor(ExportedStat::float(
        "touch",
        "average_ball_speed_change",
        StatUnit::UnrealUnitsPerSecond,
        stats.average_ball_speed_change(),
    ));
    visitor(ExportedStat::float(
        "touch",
        "max_ball_speed_change",
        StatUnit::UnrealUnitsPerSecond,
        stats.max_ball_speed_change,
    ));
    visitor(ExportedStat::float(
        "touch",
        "total_ball_travel_distance",
        StatUnit::UnrealUnits,
        stats.total_ball_travel_distance,
    ));
    visitor(ExportedStat::float(
        "touch",
        "total_ball_advance_distance",
        StatUnit::UnrealUnits,
        stats.total_ball_advance_distance,
    ));
    visitor(ExportedStat::float(
        "touch",
        "total_ball_retreat_distance",
        StatUnit::UnrealUnits,
        stats.total_ball_retreat_distance,
    ));
}
