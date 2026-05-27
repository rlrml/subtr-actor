use super::*;

pub(super) fn visit_rotation_player_count_fields(
    stats: &RotationPlayerStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::unsigned(
        "rotation",
        "first_man_stints",
        StatUnit::Count,
        stats.first_man_stint_count,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "longest_first_man_stint_time",
        StatUnit::Seconds,
        stats.longest_first_man_stint_time,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "avg_first_man_stint_time",
        StatUnit::Seconds,
        stats.average_first_man_stint_time(),
    ));
    visitor(ExportedStat::unsigned(
        "rotation",
        "became_first_man",
        StatUnit::Count,
        stats.became_first_man_count,
    ));
    visitor(ExportedStat::unsigned(
        "rotation",
        "lost_first_man",
        StatUnit::Count,
        stats.lost_first_man_count,
    ));
}
