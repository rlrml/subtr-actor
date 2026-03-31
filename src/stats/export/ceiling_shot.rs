use crate::*;

use super::*;

impl StatFieldProvider for CeilingShotStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "ceiling_shot",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "ceiling_shot",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "ceiling_shot",
            "is_last_ceiling_shot",
            StatUnit::Count,
            u32::from(self.is_last_ceiling_shot),
        ));
        if let Some(value) = self.last_ceiling_shot_time {
            visitor(ExportedStat::float(
                "ceiling_shot",
                "last_ceiling_shot_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_ceiling_shot_frame {
            visitor(ExportedStat::unsigned(
                "ceiling_shot",
                "last_ceiling_shot_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_ceiling_shot {
            visitor(ExportedStat::float(
                "ceiling_shot",
                "time_since_last_ceiling_shot",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_ceiling_shot {
            visitor(ExportedStat::unsigned(
                "ceiling_shot",
                "frames_since_last_ceiling_shot",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_confidence {
            visitor(ExportedStat::float(
                "ceiling_shot",
                "last_confidence",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "ceiling_shot",
            "average_confidence",
            StatUnit::Percent,
            self.average_confidence() * 100.0,
        ));
        visitor(ExportedStat::float(
            "ceiling_shot",
            "best_confidence",
            StatUnit::Percent,
            self.best_confidence * 100.0,
        ));
    }
}
