use crate::*;

use super::*;

impl StatFieldProvider for PressureStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "pressure",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_zero_side_time",
            StatUnit::Seconds,
            self.team_zero_side_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_one_side_time",
            StatUnit::Seconds,
            self.team_one_side_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_zero_side_pct",
            StatUnit::Percent,
            self.team_zero_side_pct(),
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_one_side_pct",
            StatUnit::Percent,
            self.team_one_side_pct(),
        ));
    }
}
