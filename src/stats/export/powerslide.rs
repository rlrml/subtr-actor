use crate::*;

use super::*;

impl StatFieldProvider for PowerslideStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "powerslide",
            "time_powerslide",
            StatUnit::Seconds,
            self.total_duration,
        ));
        visitor(ExportedStat::unsigned(
            "powerslide",
            "count_powerslide",
            StatUnit::Count,
            self.press_count,
        ));
        visitor(ExportedStat::float(
            "powerslide",
            "avg_powerslide_duration",
            StatUnit::Seconds,
            self.average_duration(),
        ));
    }
}
