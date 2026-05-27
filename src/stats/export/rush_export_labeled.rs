use crate::{ExportedStat, RushStats, StatUnit};

pub(super) fn visit_labeled_rush_fields(stats: &RushStats, visitor: &mut dyn FnMut(ExportedStat)) {
    for entry in stats.complete_labeled_rush_counts().entries {
        visitor(ExportedStat::unsigned_labeled(
            "rush",
            "rush_count",
            StatUnit::Count,
            entry.labels,
            entry.count,
        ));
    }
}
