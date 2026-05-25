use crate::*;

use super::*;

impl StatFieldProvider for WallAerialShotStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "wall_aerial_shot",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "wall_aerial_shot",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "wall_aerial_shot",
            "is_last_wall_aerial_shot",
            StatUnit::Count,
            u32::from(self.is_last_wall_aerial_shot),
        ));
        if let Some(value) = self.last_wall_aerial_shot_time {
            visitor(ExportedStat::float(
                "wall_aerial_shot",
                "last_wall_aerial_shot_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_wall_aerial_shot_frame {
            visitor(ExportedStat::unsigned(
                "wall_aerial_shot",
                "last_wall_aerial_shot_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_wall_aerial_shot {
            visitor(ExportedStat::float(
                "wall_aerial_shot",
                "time_since_last_wall_aerial_shot",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_wall_aerial_shot {
            visitor(ExportedStat::unsigned(
                "wall_aerial_shot",
                "frames_since_last_wall_aerial_shot",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_confidence {
            visitor(ExportedStat::float(
                "wall_aerial_shot",
                "last_confidence",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "wall_aerial_shot",
            "average_confidence",
            StatUnit::Percent,
            self.average_confidence() * 100.0,
        ));
        visitor(ExportedStat::float(
            "wall_aerial_shot",
            "best_confidence",
            StatUnit::Percent,
            self.best_confidence * 100.0,
        ));
        visitor(ExportedStat::float(
            "wall_aerial_shot",
            "average_takeoff_to_shot_time",
            StatUnit::Seconds,
            self.average_takeoff_to_shot_time(),
        ));
        visitor(ExportedStat::float(
            "wall_aerial_shot",
            "average_shot_height",
            StatUnit::UnrealUnits,
            self.average_shot_height(),
        ));
    }
}
