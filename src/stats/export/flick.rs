use crate::*;

use super::*;

impl StatFieldProvider for FlickStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "flick",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "flick",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "flick",
            "is_last_flick",
            StatUnit::Count,
            u32::from(self.is_last_flick),
        ));
        if let Some(value) = self.last_flick_time {
            visitor(ExportedStat::float(
                "flick",
                "last_flick_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_flick_frame {
            visitor(ExportedStat::unsigned(
                "flick",
                "last_flick_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_flick {
            visitor(ExportedStat::float(
                "flick",
                "time_since_last_flick",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_flick {
            visitor(ExportedStat::unsigned(
                "flick",
                "frames_since_last_flick",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_confidence {
            visitor(ExportedStat::float(
                "flick",
                "last_confidence",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "flick",
            "average_confidence",
            StatUnit::Percent,
            self.average_confidence() * 100.0,
        ));
        visitor(ExportedStat::float(
            "flick",
            "best_confidence",
            StatUnit::Percent,
            self.best_confidence * 100.0,
        ));
        visitor(ExportedStat::float(
            "flick",
            "average_setup_duration",
            StatUnit::Seconds,
            self.average_setup_duration(),
        ));
        visitor(ExportedStat::float(
            "flick",
            "average_ball_speed_change",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed_change(),
        ));
    }
}
