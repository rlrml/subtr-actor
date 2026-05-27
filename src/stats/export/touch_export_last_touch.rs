use crate::{ExportedStat, StatUnit, TouchStats};

pub(super) fn visit_last_touch_fields(stats: &TouchStats, visitor: &mut dyn FnMut(ExportedStat)) {
    visitor(ExportedStat::unsigned(
        "touch",
        "is_last_touch",
        StatUnit::Count,
        u32::from(stats.is_last_touch),
    ));
    if let Some(value) = stats.last_touch_time {
        visitor(ExportedStat::float(
            "touch",
            "last_touch_time",
            StatUnit::Seconds,
            value,
        ));
    }
    if let Some(value) = stats.last_touch_frame {
        visitor(ExportedStat::unsigned(
            "touch",
            "last_touch_frame",
            StatUnit::Count,
            u32::try_from(value).unwrap_or(u32::MAX),
        ));
    }
    if let Some(value) = stats.time_since_last_touch {
        visitor(ExportedStat::float(
            "touch",
            "time_since_last_touch",
            StatUnit::Seconds,
            value,
        ));
    }
    if let Some(value) = stats.frames_since_last_touch {
        visitor(ExportedStat::unsigned(
            "touch",
            "frames_since_last_touch",
            StatUnit::Count,
            u32::try_from(value).unwrap_or(u32::MAX),
        ));
    }
}
