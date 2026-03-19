use crate::*;

use super::*;

impl StatFieldProvider for SpeedFlipStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "speed_flip",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "speed_flip",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "speed_flip",
            "is_last_speed_flip",
            StatUnit::Count,
            u32::from(self.is_last_speed_flip),
        ));
        if let Some(value) = self.last_speed_flip_time {
            visitor(ExportedStat::float(
                "speed_flip",
                "last_speed_flip_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_speed_flip_frame {
            visitor(ExportedStat::unsigned(
                "speed_flip",
                "last_speed_flip_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_speed_flip {
            visitor(ExportedStat::float(
                "speed_flip",
                "time_since_last_speed_flip",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_speed_flip {
            visitor(ExportedStat::unsigned(
                "speed_flip",
                "frames_since_last_speed_flip",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_quality {
            visitor(ExportedStat::float(
                "speed_flip",
                "last_quality",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "speed_flip",
            "average_quality",
            StatUnit::Percent,
            self.average_quality() * 100.0,
        ));
        visitor(ExportedStat::float(
            "speed_flip",
            "best_quality",
            StatUnit::Percent,
            self.best_quality * 100.0,
        ));
    }
}
