use crate::*;

use super::*;

impl StatFieldProvider for PassPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "pass",
            "completed_pass_count",
            StatUnit::Count,
            self.completed_pass_count,
        ));
        visitor(ExportedStat::unsigned(
            "pass",
            "received_pass_count",
            StatUnit::Count,
            self.received_pass_count,
        ));
        visitor(ExportedStat::float(
            "pass",
            "average_pass_distance",
            StatUnit::UnrealUnits,
            self.average_pass_distance(),
        ));
        visitor(ExportedStat::float(
            "pass",
            "average_pass_advance",
            StatUnit::UnrealUnits,
            self.average_pass_advance(),
        ));
        visitor(ExportedStat::float(
            "pass",
            "longest_pass_distance",
            StatUnit::UnrealUnits,
            self.longest_pass_distance,
        ));
    }
}

impl StatFieldProvider for PassTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "pass",
            "completed_pass_count",
            StatUnit::Count,
            self.completed_pass_count,
        ));
        visitor(ExportedStat::float(
            "pass",
            "average_pass_distance",
            StatUnit::UnrealUnits,
            self.average_pass_distance(),
        ));
        visitor(ExportedStat::float(
            "pass",
            "average_pass_advance",
            StatUnit::UnrealUnits,
            self.average_pass_advance(),
        ));
        visitor(ExportedStat::float(
            "pass",
            "longest_pass_distance",
            StatUnit::UnrealUnits,
            self.longest_pass_distance,
        ));
    }
}
