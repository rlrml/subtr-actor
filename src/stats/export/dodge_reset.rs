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
    }
}
