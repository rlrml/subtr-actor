use crate::*;

use super::*;

impl StatFieldProvider for WhiffStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "whiff",
            "whiff_count",
            StatUnit::Count,
            self.whiff_count,
        ));
        visitor(ExportedStat::unsigned(
            "whiff",
            "grounded_whiff_count",
            StatUnit::Count,
            self.grounded_whiff_count,
        ));
        visitor(ExportedStat::unsigned(
            "whiff",
            "aerial_whiff_count",
            StatUnit::Count,
            self.aerial_whiff_count,
        ));
        visitor(ExportedStat::unsigned(
            "whiff",
            "dodge_whiff_count",
            StatUnit::Count,
            self.dodge_whiff_count,
        ));
        visitor(ExportedStat::unsigned(
            "whiff",
            "is_last_whiff",
            StatUnit::Count,
            u32::from(self.is_last_whiff),
        ));
        if let Some(value) = self.last_whiff_time {
            visitor(ExportedStat::float(
                "whiff",
                "last_whiff_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_whiff_frame {
            visitor(ExportedStat::unsigned(
                "whiff",
                "last_whiff_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_whiff {
            visitor(ExportedStat::float(
                "whiff",
                "time_since_last_whiff",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_whiff {
            visitor(ExportedStat::unsigned(
                "whiff",
                "frames_since_last_whiff",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_closest_approach_distance {
            visitor(ExportedStat::float(
                "whiff",
                "last_closest_approach_distance",
                StatUnit::UnrealUnits,
                value,
            ));
        }
        if let Some(value) = self.best_closest_approach_distance {
            visitor(ExportedStat::float(
                "whiff",
                "best_closest_approach_distance",
                StatUnit::UnrealUnits,
                value,
            ));
        }
        visitor(ExportedStat::float(
            "whiff",
            "average_closest_approach_distance",
            StatUnit::UnrealUnits,
            self.average_closest_approach_distance(),
        ));
    }
}
