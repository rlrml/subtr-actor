use super::*;

pub(super) fn visit_goal_timing(stats: &CorePlayerStats, visitor: &mut dyn FnMut(ExportedStat)) {
    visitor(ExportedStat::float(
        "core",
        "average_goal_time_after_kickoff",
        StatUnit::Seconds,
        stats.average_goal_time_after_kickoff(),
    ));
    visitor(ExportedStat::float(
        "core",
        "median_goal_time_after_kickoff",
        StatUnit::Seconds,
        stats.median_goal_time_after_kickoff(),
    ));
    visitor(ExportedStat::unsigned(
        "core",
        "goal_ball_air_time_sample_count",
        StatUnit::Count,
        stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_time_sample_count,
    ));
    visitor(ExportedStat::float(
        "core",
        "average_goal_ball_air_time",
        StatUnit::Seconds,
        stats.average_goal_ball_air_time(),
    ));
    visitor(ExportedStat::float(
        "core",
        "median_goal_ball_air_time",
        StatUnit::Seconds,
        stats.median_goal_ball_air_time(),
    ));
}
