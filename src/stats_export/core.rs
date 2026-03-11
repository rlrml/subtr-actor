use crate::*;

use super::*;

impl StatFieldProvider for CorePlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::signed(
            "core",
            "score",
            StatUnit::Count,
            self.score,
        ));
        visitor(ExportedStat::signed(
            "core",
            "goals",
            StatUnit::Count,
            self.goals,
        ));
        visitor(ExportedStat::signed(
            "core",
            "assists",
            StatUnit::Count,
            self.assists,
        ));
        visitor(ExportedStat::signed(
            "core",
            "saves",
            StatUnit::Count,
            self.saves,
        ));
        visitor(ExportedStat::signed(
            "core",
            "shots",
            StatUnit::Count,
            self.shots,
        ));
        visitor(ExportedStat::unsigned(
            "core",
            "goals_conceded_while_last_defender",
            StatUnit::Count,
            self.goals_conceded_while_last_defender,
        ));
        visitor(ExportedStat::float(
            "core",
            "shooting_percentage",
            StatUnit::Percent,
            self.shooting_percentage(),
        ));
    }
}

impl StatFieldProvider for CoreTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::signed(
            "core",
            "score",
            StatUnit::Count,
            self.score,
        ));
        visitor(ExportedStat::signed(
            "core",
            "goals",
            StatUnit::Count,
            self.goals,
        ));
        visitor(ExportedStat::signed(
            "core",
            "assists",
            StatUnit::Count,
            self.assists,
        ));
        visitor(ExportedStat::signed(
            "core",
            "saves",
            StatUnit::Count,
            self.saves,
        ));
        visitor(ExportedStat::signed(
            "core",
            "shots",
            StatUnit::Count,
            self.shots,
        ));
        visitor(ExportedStat::float(
            "core",
            "shooting_percentage",
            StatUnit::Percent,
            self.shooting_percentage(),
        ));
    }
}
