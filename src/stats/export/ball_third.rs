use crate::*;

use super::*;

impl StatFieldProvider for BallThirdStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "ball_third",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        for entry in &self.labeled_time.entries {
            visitor(ExportedStat::float_labeled(
                "ball_third",
                "time",
                StatUnit::Seconds,
                entry.labels.clone(),
                entry.value,
            ));
        }
        visitor(ExportedStat::float(
            "ball_third",
            "team_zero_third_time",
            StatUnit::Seconds,
            self.team_zero_third_time,
        ));
        visitor(ExportedStat::float(
            "ball_third",
            "neutral_third_time",
            StatUnit::Seconds,
            self.neutral_third_time,
        ));
        visitor(ExportedStat::float(
            "ball_third",
            "team_one_third_time",
            StatUnit::Seconds,
            self.team_one_third_time,
        ));
        visitor(ExportedStat::float(
            "ball_third",
            "team_zero_third_pct",
            StatUnit::Percent,
            self.team_zero_third_pct(),
        ));
        visitor(ExportedStat::float(
            "ball_third",
            "neutral_third_pct",
            StatUnit::Percent,
            self.neutral_third_pct(),
        ));
        visitor(ExportedStat::float(
            "ball_third",
            "team_one_third_pct",
            StatUnit::Percent,
            self.team_one_third_pct(),
        ));
    }
}

#[cfg(test)]
#[path = "ball_third_test.rs"]
mod tests;
