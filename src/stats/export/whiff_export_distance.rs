use crate::{ExportedStat, StatUnit, WhiffStats};

pub(super) fn visit_whiff_distance_fields(
    stats: &WhiffStats,
    visitor: &mut dyn FnMut(ExportedStat),
) {
    if let Some(value) = stats.last_closest_approach_distance {
        visitor(ExportedStat::float(
            "whiff",
            "last_closest_approach_distance",
            StatUnit::UnrealUnits,
            value,
        ));
    }
    if let Some(value) = stats.best_closest_approach_distance {
        visitor(ExportedStat::float(
            "whiff",
            "best_closest_approach_distance",
            StatUnit::UnrealUnits,
            value,
        ));
    }
    visitor(ExportedStat::float(
        "whiff",
        "average_closest_approach_distance",
        StatUnit::UnrealUnits,
        stats.average_closest_approach_distance(),
    ));
}
