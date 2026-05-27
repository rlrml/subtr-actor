use crate::{ExportedStat, MovementStats, StatUnit};

pub(super) fn visit_movement_percent_fields(
    stats: &MovementStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "movement",
        "avg_speed_percentage",
        StatUnit::Percent,
        stats.average_speed_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_slow_speed",
        StatUnit::Percent,
        stats.slow_speed_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_boost_speed",
        StatUnit::Percent,
        stats.boost_speed_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_supersonic_speed",
        StatUnit::Percent,
        stats.supersonic_speed_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_ground",
        StatUnit::Percent,
        stats.on_ground_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_low_air",
        StatUnit::Percent,
        stats.low_air_pct(),
    ));
    visitor(ExportedStat::float(
        "movement",
        "percent_high_air",
        StatUnit::Percent,
        stats.high_air_pct(),
    ));
}
