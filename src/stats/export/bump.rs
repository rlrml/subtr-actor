use crate::*;

use super::*;

impl StatFieldProvider for BumpPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "bump",
            "inflicted",
            StatUnit::Count,
            self.bumps_inflicted,
        ));
        visitor(ExportedStat::unsigned(
            "bump",
            "taken",
            StatUnit::Count,
            self.bumps_taken,
        ));
        visitor(ExportedStat::unsigned(
            "bump",
            "team_inflicted",
            StatUnit::Count,
            self.team_bumps_inflicted,
        ));
        visitor(ExportedStat::unsigned(
            "bump",
            "team_taken",
            StatUnit::Count,
            self.team_bumps_taken,
        ));
    }
}

impl StatFieldProvider for BumpTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "bump",
            "inflicted",
            StatUnit::Count,
            self.bumps_inflicted,
        ));
        visitor(ExportedStat::unsigned(
            "bump",
            "team_inflicted",
            StatUnit::Count,
            self.team_bumps_inflicted,
        ));
    }
}
