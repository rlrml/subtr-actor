use crate::*;

use super::*;

impl StatFieldProvider for HalfFlipStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "half_flip",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "half_flip",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "half_flip",
            "is_last_half_flip",
            StatUnit::Count,
            u32::from(self.is_last_half_flip),
        ));
        if let Some(value) = self.last_half_flip_time {
            visitor(ExportedStat::float(
                "half_flip",
                "last_half_flip_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_half_flip_frame {
            visitor(ExportedStat::unsigned(
                "half_flip",
                "last_half_flip_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_half_flip {
            visitor(ExportedStat::float(
                "half_flip",
                "time_since_last_half_flip",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_half_flip {
            visitor(ExportedStat::unsigned(
                "half_flip",
                "frames_since_last_half_flip",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_quality {
            visitor(ExportedStat::float(
                "half_flip",
                "last_quality",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "half_flip",
            "average_quality",
            StatUnit::Percent,
            self.average_quality() * 100.0,
        ));
        visitor(ExportedStat::float(
            "half_flip",
            "best_quality",
            StatUnit::Percent,
            self.best_quality * 100.0,
        ));
        for entry in self.complete_labeled_event_counts().entries {
            visitor(ExportedStat::unsigned_labeled(
                "half_flip",
                "count",
                StatUnit::Count,
                entry.labels,
                entry.count,
            ));
        }
    }
}
