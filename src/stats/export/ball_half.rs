use crate::*;

use super::*;

impl StatFieldProvider for BallHalfStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "ball_half",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        for entry in &self.labeled_time.entries {
            visitor(ExportedStat::float_labeled(
                "ball_half",
                "time",
                StatUnit::Seconds,
                entry.labels.clone(),
                entry.value,
            ));
        }
        visitor(ExportedStat::float(
            "ball_half",
            "team_zero_side_time",
            StatUnit::Seconds,
            self.team_zero_side_time,
        ));
        visitor(ExportedStat::float(
            "ball_half",
            "team_one_side_time",
            StatUnit::Seconds,
            self.team_one_side_time,
        ));
        visitor(ExportedStat::float(
            "ball_half",
            "neutral_time",
            StatUnit::Seconds,
            self.neutral_time,
        ));
        visitor(ExportedStat::float(
            "ball_half",
            "team_zero_side_pct",
            StatUnit::Percent,
            self.team_zero_side_pct(),
        ));
        visitor(ExportedStat::float(
            "ball_half",
            "team_one_side_pct",
            StatUnit::Percent,
            self.team_one_side_pct(),
        ));
        visitor(ExportedStat::float(
            "ball_half",
            "neutral_pct",
            StatUnit::Percent,
            self.neutral_pct(),
        ));
    }
}

#[cfg(test)]
#[path = "ball_half_test.rs"]
mod tests;
