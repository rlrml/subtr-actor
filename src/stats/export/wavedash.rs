use crate::*;

use super::*;

impl StatFieldProvider for WavedashStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "wavedash",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "wavedash",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "wavedash",
            "is_last_wavedash",
            StatUnit::Count,
            u32::from(self.is_last_wavedash),
        ));
        if let Some(value) = self.last_wavedash_time {
            visitor(ExportedStat::float(
                "wavedash",
                "last_wavedash_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_wavedash_frame {
            visitor(ExportedStat::unsigned(
                "wavedash",
                "last_wavedash_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_wavedash {
            visitor(ExportedStat::float(
                "wavedash",
                "time_since_last_wavedash",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_wavedash {
            visitor(ExportedStat::unsigned(
                "wavedash",
                "frames_since_last_wavedash",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_quality {
            visitor(ExportedStat::float(
                "wavedash",
                "last_quality",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "wavedash",
            "average_quality",
            StatUnit::Percent,
            self.average_quality() * 100.0,
        ));
        visitor(ExportedStat::float(
            "wavedash",
            "best_quality",
            StatUnit::Percent,
            self.best_quality * 100.0,
        ));
        for entry in self.complete_labeled_event_counts().entries {
            visitor(ExportedStat::unsigned_labeled(
                "wavedash",
                "count",
                StatUnit::Count,
                entry.labels,
                entry.count,
            ));
        }
    }
}
