use super::*;

pub(super) fn visit_rotation_player_time_fields(
    stats: &RotationPlayerStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "rotation",
        "active_game_time",
        StatUnit::Seconds,
        stats.active_game_time,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_first_man",
        StatUnit::Seconds,
        stats.time_first_man,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_second_man",
        StatUnit::Seconds,
        stats.time_second_man,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_third_man",
        StatUnit::Seconds,
        stats.time_third_man,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_ambiguous_role",
        StatUnit::Seconds,
        stats.time_ambiguous_role,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_behind_play",
        StatUnit::Seconds,
        stats.time_behind_play,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_level_with_play",
        StatUnit::Seconds,
        stats.time_level_with_play,
    ));
    visitor(ExportedStat::float(
        "rotation",
        "time_ahead_of_play",
        StatUnit::Seconds,
        stats.time_ahead_of_play,
    ));
}
