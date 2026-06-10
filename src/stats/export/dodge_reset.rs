use crate::*;

use super::*;

impl StatFieldProvider for DodgeResetStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "dodge_reset",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::unsigned(
            "dodge_reset",
            "on_ball_count",
            StatUnit::Count,
            self.on_ball_count,
        ));
        visitor(ExportedStat::unsigned(
            "dodge_reset",
            "flip_reset_used_count",
            StatUnit::Count,
            self.flip_reset_used_count,
        ));
        visitor(ExportedStat::unsigned(
            "dodge_reset",
            "flip_reset_unused_count",
            StatUnit::Count,
            self.flip_reset_unused_count,
        ));
        visitor(ExportedStat::float(
            "dodge_reset",
            "flip_reset_mean_time_to_use",
            StatUnit::Seconds,
            self.flip_reset_mean_time_to_use(),
        ));
        visitor(ExportedStat::float(
            "dodge_reset",
            "flip_reset_min_time_to_use",
            StatUnit::Seconds,
            self.flip_reset_min_time_to_use.unwrap_or(0.0),
        ));
    }
}
