use crate::*;

use super::*;

impl StatFieldProvider for DemoPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "demo",
            "inflicted",
            StatUnit::Count,
            self.demos_inflicted,
        ));
        visitor(ExportedStat::unsigned(
            "demo",
            "taken",
            StatUnit::Count,
            self.demos_taken,
        ));
    }
}

impl StatFieldProvider for DemoTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "demo",
            "inflicted",
            StatUnit::Count,
            self.demos_inflicted,
        ));
    }
}
