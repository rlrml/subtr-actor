use crate::*;

use super::*;

impl StatFieldProvider for MustyFlickStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "musty_flick",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "musty_flick",
            "aerial_count",
            StatUnit::Count,
            self.aerial_count,
        ));
        visitor(ExportedStat::unsigned(
            "musty_flick",
            "high_confidence_count",
            StatUnit::Count,
            self.high_confidence_count,
        ));
        visitor(ExportedStat::unsigned(
            "musty_flick",
            "is_last_musty",
            StatUnit::Count,
            u32::from(self.is_last_musty),
        ));
        if let Some(value) = self.last_musty_time {
            visitor(ExportedStat::float(
                "musty_flick",
                "last_musty_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_musty_frame {
            visitor(ExportedStat::unsigned(
                "musty_flick",
                "last_musty_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_musty {
            visitor(ExportedStat::float(
                "musty_flick",
                "time_since_last_musty",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_musty {
            visitor(ExportedStat::unsigned(
                "musty_flick",
                "frames_since_last_musty",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_confidence {
            visitor(ExportedStat::float(
                "musty_flick",
                "last_confidence",
                StatUnit::Percent,
                value * 100.0,
            ));
        }
        visitor(ExportedStat::float(
            "musty_flick",
            "average_confidence",
            StatUnit::Percent,
            self.average_confidence() * 100.0,
        ));
        visitor(ExportedStat::float(
            "musty_flick",
            "best_confidence",
            StatUnit::Percent,
            self.best_confidence * 100.0,
        ));
    }
}
