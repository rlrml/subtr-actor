use crate::{AirDribbleStats, ExportedStat, StatUnit};

pub(super) fn visit_air_dribble_count_fields(
    stats: &AirDribbleStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::unsigned(
        "air_dribble",
        "count",
        StatUnit::Count,
        stats.count,
    ));
    visitor(ExportedStat::unsigned(
        "air_dribble",
        "ground_to_air_count",
        StatUnit::Count,
        stats.ground_to_air_count,
    ));
    visitor(ExportedStat::unsigned(
        "air_dribble",
        "wall_to_air_count",
        StatUnit::Count,
        stats.wall_to_air_count,
    ));
    visitor(ExportedStat::unsigned(
        "air_dribble",
        "total_touch_count",
        StatUnit::Count,
        stats.total_touch_count,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_touch_count",
        StatUnit::Count,
        stats.average_touch_count(),
    ));
    visitor(ExportedStat::unsigned(
        "air_dribble",
        "max_touch_count",
        StatUnit::Count,
        stats.max_touch_count,
    ));
    for entry in stats.complete_labeled_event_counts().entries {
        visitor(ExportedStat::unsigned_labeled(
            "air_dribble",
            "count",
            StatUnit::Count,
            entry.labels,
            entry.count,
        ));
    }
}
