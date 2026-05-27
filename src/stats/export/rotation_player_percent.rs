use super::*;

pub(super) fn visit_rotation_player_percent_fields(
    stats: &RotationPlayerStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "rotation",
        "percent_first_man",
        StatUnit::Percent,
        stats.first_man_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_second_man",
        StatUnit::Percent,
        stats.second_man_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_third_man",
        StatUnit::Percent,
        stats.third_man_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_ambiguous_role",
        StatUnit::Percent,
        stats.ambiguous_role_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_behind_play",
        StatUnit::Percent,
        stats.behind_play_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_level_with_play",
        StatUnit::Percent,
        stats.level_with_play_pct(),
    ));
    visitor(ExportedStat::float(
        "rotation",
        "percent_ahead_of_play",
        StatUnit::Percent,
        stats.ahead_of_play_pct(),
    ));
}
