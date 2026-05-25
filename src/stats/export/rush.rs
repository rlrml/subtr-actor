use crate::*;

use super::*;

impl StatFieldProvider for RushStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_count",
            StatUnit::Count,
            self.team_zero_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_two_v_one_count",
            StatUnit::Count,
            self.team_zero_two_v_one_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_two_v_two_count",
            StatUnit::Count,
            self.team_zero_two_v_two_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_two_v_three_count",
            StatUnit::Count,
            self.team_zero_two_v_three_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_three_v_one_count",
            StatUnit::Count,
            self.team_zero_three_v_one_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_three_v_two_count",
            StatUnit::Count,
            self.team_zero_three_v_two_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_zero_three_v_three_count",
            StatUnit::Count,
            self.team_zero_three_v_three_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_count",
            StatUnit::Count,
            self.team_one_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_two_v_one_count",
            StatUnit::Count,
            self.team_one_two_v_one_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_two_v_two_count",
            StatUnit::Count,
            self.team_one_two_v_two_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_two_v_three_count",
            StatUnit::Count,
            self.team_one_two_v_three_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_three_v_one_count",
            StatUnit::Count,
            self.team_one_three_v_one_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_three_v_two_count",
            StatUnit::Count,
            self.team_one_three_v_two_count,
        ));
        visitor(ExportedStat::unsigned(
            "rush",
            "team_one_three_v_three_count",
            StatUnit::Count,
            self.team_one_three_v_three_count,
        ));
        for entry in self.complete_labeled_rush_counts().entries {
            visitor(ExportedStat::unsigned_labeled(
                "rush",
                "rush_count",
                StatUnit::Count,
                entry.labels,
                entry.count,
            ));
        }
    }
}

#[cfg(test)]
#[path = "rush_test.rs"]
mod tests;
