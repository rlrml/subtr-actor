use crate::*;

use super::*;

impl StatFieldProvider for DoubleTapPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "double_tap",
            "count",
            StatUnit::Count,
            self.count,
        ));
    }
}

impl StatFieldProvider for DoubleTapTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "double_tap",
            "count",
            StatUnit::Count,
            self.count,
        ));
    }
}
