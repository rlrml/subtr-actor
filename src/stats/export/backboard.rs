use crate::*;

use super::*;

impl StatFieldProvider for BackboardPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "backboard",
            "count",
            StatUnit::Count,
            self.count,
        ));
    }
}

impl StatFieldProvider for BackboardTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "backboard",
            "count",
            StatUnit::Count,
            self.count,
        ));
    }
}
