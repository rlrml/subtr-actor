use crate::{ExportedStat, StatUnit, WhiffStats};

pub(super) fn visit_whiff_count_fields(stats: &WhiffStats, visitor: &mut dyn FnMut(ExportedStat)) {
    visitor(ExportedStat::unsigned(
        "whiff",
        "whiff_count",
        StatUnit::Count,
        stats.whiff_count,
    ));
    visitor(ExportedStat::unsigned(
        "whiff",
        "beaten_to_ball_count",
        StatUnit::Count,
        stats.beaten_to_ball_count,
    ));
    visitor(ExportedStat::unsigned(
        "whiff",
        "grounded_whiff_count",
        StatUnit::Count,
        stats.grounded_whiff_count,
    ));
    visitor(ExportedStat::unsigned(
        "whiff",
        "aerial_whiff_count",
        StatUnit::Count,
        stats.aerial_whiff_count,
    ));
    visitor(ExportedStat::unsigned(
        "whiff",
        "dodge_whiff_count",
        StatUnit::Count,
        stats.dodge_whiff_count,
    ));
    for entry in stats.complete_labeled_whiff_counts().entries {
        visitor(ExportedStat::unsigned_labeled(
            "whiff",
            "whiff_count",
            StatUnit::Count,
            entry.labels,
            entry.count,
        ));
    }
}
