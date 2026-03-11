use crate::*;

use super::*;

impl StatFieldProvider for PossessionStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "possession",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_zero_time",
            StatUnit::Seconds,
            self.team_zero_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_time",
            StatUnit::Seconds,
            self.team_one_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_zero_pct",
            StatUnit::Percent,
            self.team_zero_pct(),
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_pct",
            StatUnit::Percent,
            self.team_one_pct(),
        ));
    }
}
