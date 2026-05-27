use crate::{ExportedStat, StatUnit, WhiffStats};

pub(super) fn visit_last_whiff_fields(stats: &WhiffStats, visitor: &mut dyn FnMut(ExportedStat)) {
    visitor(ExportedStat::unsigned(
        "whiff",
        "is_last_whiff",
        StatUnit::Count,
        u32::from(stats.is_last_whiff),
    ));
    if let Some(value) = stats.last_whiff_time {
        visitor(ExportedStat::float(
            "whiff",
            "last_whiff_time",
            StatUnit::Seconds,
            value,
        ));
    }
    if let Some(value) = stats.last_whiff_frame {
        visitor(ExportedStat::unsigned(
            "whiff",
            "last_whiff_frame",
            StatUnit::Count,
            u32::try_from(value).unwrap_or(u32::MAX),
        ));
    }
    if let Some(value) = stats.time_since_last_whiff {
        visitor(ExportedStat::float(
            "whiff",
            "time_since_last_whiff",
            StatUnit::Seconds,
            value,
        ));
    }
    if let Some(value) = stats.frames_since_last_whiff {
        visitor(ExportedStat::unsigned(
            "whiff",
            "frames_since_last_whiff",
            StatUnit::Count,
            u32::try_from(value).unwrap_or(u32::MAX),
        ));
    }
}
