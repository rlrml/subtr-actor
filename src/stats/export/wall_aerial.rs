use crate::*;

use super::*;

impl StatFieldProvider for WallAerialStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "wall_aerial",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "wall_aerial",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "wall_aerial",
            "is_last_wall_aerial",
            StatUnit::Count,
            u32::from(self.is_last_wall_aerial),
        ));
        if let Some(value) = self.last_wall_aerial_time {
            visitor(ExportedStat::float(
                "wall_aerial",
                "last_wall_aerial_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_wall_aerial_frame {
            visitor(ExportedStat::unsigned(
                "wall_aerial",
                "last_wall_aerial_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_wall_aerial {
            visitor(ExportedStat::float(
                "wall_aerial",
                "time_since_last_wall_aerial",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_wall_aerial {
            visitor(ExportedStat::unsigned(
                "wall_aerial",
                "frames_since_last_wall_aerial",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_confidence {
            visitor(ExportedStat::float(
                "wall_aerial",
                "last_confidence",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "wall_aerial",
            "average_confidence",
            StatUnit::Percent,
            self.average_confidence() * 100.0,
        ));
        visitor(ExportedStat::float(
            "wall_aerial",
            "best_confidence",
            StatUnit::Percent,
            self.best_confidence * 100.0,
        ));
        visitor(ExportedStat::float(
            "wall_aerial",
            "average_setup_duration",
            StatUnit::Seconds,
            self.average_setup_duration(),
        ));
        visitor(ExportedStat::float(
            "wall_aerial",
            "average_takeoff_to_touch_time",
            StatUnit::Seconds,
            self.average_takeoff_to_touch_time(),
        ));
        visitor(ExportedStat::float(
            "wall_aerial",
            "average_touch_height",
            StatUnit::UnrealUnits,
            self.average_touch_height(),
        ));
    }
}
