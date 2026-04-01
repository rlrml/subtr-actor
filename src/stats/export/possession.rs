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
        for entry in &self.labeled_time.entries {
            visitor(ExportedStat::float_labeled(
                "possession",
                "time",
                StatUnit::Seconds,
                entry.labels.clone(),
                entry.value,
            ));
        }
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
            "neutral_time",
            StatUnit::Seconds,
            self.neutral_time,
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
        visitor(ExportedStat::float(
            "possession",
            "neutral_pct",
            StatUnit::Percent,
            self.neutral_pct(),
        ));
    }
}

#[cfg(test)]
#[path = "possession_test.rs"]
mod tests;
