use super::*;

pub(super) fn visit_team_percent_fields(
    stats: &FiftyFiftyStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    visitor(ExportedStat::float(
        "fifty_fifty",
        "team_zero_win_pct",
        StatUnit::Percent,
        stats.team_zero_win_pct(),
    ));
    visitor(ExportedStat::float(
        "fifty_fifty",
        "team_one_win_pct",
        StatUnit::Percent,
        stats.team_one_win_pct(),
    ));
    visitor(ExportedStat::float(
        "fifty_fifty",
        "kickoff_team_zero_win_pct",
        StatUnit::Percent,
        stats.kickoff_team_zero_win_pct(),
    ));
    visitor(ExportedStat::float(
        "fifty_fifty",
        "kickoff_team_one_win_pct",
        StatUnit::Percent,
        stats.kickoff_team_one_win_pct(),
    ));
}

pub(super) fn visit_team_labeled_count_fields(
    stats: &FiftyFiftyStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    for entry in stats.complete_labeled_event_counts().entries {
        visitor(ExportedStat::unsigned_labeled(
            "fifty_fifty",
            "count",
            StatUnit::Count,
            entry.labels,
            entry.count,
        ));
    }
}
