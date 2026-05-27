use crate::{AirDribbleStats, ExportedStat, StatUnit};

pub(super) fn visit_air_dribble_time_fields(
    stats: &AirDribbleStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "air_dribble",
        "total_time",
        StatUnit::Seconds,
        stats.total_time,
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "avg_time",
        StatUnit::Seconds,
        stats.average_time(),
    ));
    visitor(ExportedStat::float(
        "air_dribble",
        "longest_time",
        StatUnit::Seconds,
        stats.longest_time,
    ));
}
