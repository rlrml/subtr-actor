use crate::{ExportedStat, MovementStats, StatUnit};

pub(super) fn visit_movement_time_fields(
    stats: &MovementStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "movement",
        "time_supersonic_speed",
        StatUnit::Seconds,
        stats.time_supersonic_speed,
    ));
    visitor(ExportedStat::float(
        "movement",
        "time_boost_speed",
        StatUnit::Seconds,
        stats.time_boost_speed,
    ));
    visitor(ExportedStat::float(
        "movement",
        "time_slow_speed",
        StatUnit::Seconds,
        stats.time_slow_speed,
    ));
    visitor(ExportedStat::float(
        "movement",
        "time_ground",
        StatUnit::Seconds,
        stats.time_on_ground,
    ));
    visitor(ExportedStat::float(
        "movement",
        "time_low_air",
        StatUnit::Seconds,
        stats.time_low_air,
    ));
    visitor(ExportedStat::float(
        "movement",
        "time_high_air",
        StatUnit::Seconds,
        stats.time_high_air,
    ));
}
