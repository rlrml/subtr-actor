use super::*;

pub(super) fn visit_goal_against_position(
    stats: &CorePlayerStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::unsigned(
        "core",
        "goal_against_position_sample_count",
        StatUnit::Count,
        stats.scoring_context.goal_against_position_sample_count,
    ));
    visitor(ExportedStat::float(
        "core",
        "average_goal_against_position_x",
        StatUnit::UnrealUnits,
        stats.average_goal_against_position_x(),
    ));
    visitor(ExportedStat::float(
        "core",
        "average_goal_against_position_y",
        StatUnit::UnrealUnits,
        stats.average_goal_against_position_y(),
    ));
    visitor(ExportedStat::float(
        "core",
        "average_goal_against_position_z",
        StatUnit::UnrealUnits,
        stats.average_goal_against_position_z(),
    ));
}

pub(super) fn visit_scoring_touch_position(
    stats: &CorePlayerStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::unsigned(
        "core",
        "scoring_goal_last_touch_position_sample_count",
        StatUnit::Count,
        stats
            .scoring_context
            .scoring_goal_last_touch_position_sample_count,
    ));
    visitor(ExportedStat::float(
        "core",
        "average_scoring_goal_last_touch_position_x",
        StatUnit::UnrealUnits,
        stats.average_scoring_goal_last_touch_position_x(),
    ));
    visitor(ExportedStat::float(
        "core",
        "average_scoring_goal_last_touch_position_y",
        StatUnit::UnrealUnits,
        stats.average_scoring_goal_last_touch_position_y(),
    ));
    visitor(ExportedStat::float(
        "core",
        "average_scoring_goal_last_touch_position_z",
        StatUnit::UnrealUnits,
        stats.average_scoring_goal_last_touch_position_z(),
    ));
}
