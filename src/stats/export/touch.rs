use crate::*;

use super::*;

impl StatFieldProvider for TouchStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "touch",
            "touch_count",
            StatUnit::Count,
            self.touch_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "is_last_touch",
            StatUnit::Count,
            u32::from(self.is_last_touch),
        ));
        if let Some(value) = self.last_touch_time {
            visitor(ExportedStat::float(
                "touch",
                "last_touch_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_touch_frame {
            visitor(ExportedStat::unsigned(
                "touch",
                "last_touch_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_touch {
            visitor(ExportedStat::float(
                "touch",
                "time_since_last_touch",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_touch {
            visitor(ExportedStat::unsigned(
                "touch",
                "frames_since_last_touch",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
    }
}
