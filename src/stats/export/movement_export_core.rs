use crate::{ExportedStat, MovementStats, StatUnit};

pub(super) fn visit_movement_core_fields(
    stats: &MovementStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "movement",
        "tracked_time",
        StatUnit::Seconds,
        stats.tracked_time,
    ));
    for entry in stats.complete_labeled_tracked_time().entries {
        visitor(ExportedStat::float_labeled(
            "movement",
            "tracked_time",
            StatUnit::Seconds,
            entry.labels,
            entry.value,
        ));
    }
    visitor(ExportedStat::float(
        "movement",
        "total_distance",
        StatUnit::UnrealUnits,
        stats.total_distance,
    ));
    visitor(ExportedStat::float(
        "movement",
        "avg_speed",
        StatUnit::UnrealUnitsPerSecond,
        stats.average_speed(),
    ));
}
