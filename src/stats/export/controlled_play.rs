use crate::*;

use super::*;

impl StatFieldProvider for ControlledPlayStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "controlled_play",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "controlled_play",
            "total_time",
            StatUnit::Seconds,
            self.total_time,
        ));
        visitor(ExportedStat::float(
            "controlled_play",
            "avg_time",
            StatUnit::Seconds,
            self.avg_time(),
        ));
        visitor(ExportedStat::float(
            "controlled_play",
            "longest_time",
            StatUnit::Seconds,
            self.longest_time,
        ));
        visitor(ExportedStat::unsigned(
            "controlled_play",
            "touch_count",
            StatUnit::Count,
            self.touch_count,
        ));
        visitor(ExportedStat::float(
            "controlled_play",
            "total_advance_distance",
            StatUnit::UnrealUnits,
            self.total_advance_distance,
        ));
    }
}
