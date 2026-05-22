use crate::*;

use super::*;

impl StatFieldProvider for OneTimerPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "one_timer",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "one_timer",
            "average_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed(),
        ));
        visitor(ExportedStat::float(
            "one_timer",
            "fastest_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.fastest_ball_speed,
        ));
        visitor(ExportedStat::float(
            "one_timer",
            "average_pass_distance",
            StatUnit::UnrealUnits,
            self.average_pass_distance(),
        ));
    }
}

impl StatFieldProvider for OneTimerTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "one_timer",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "one_timer",
            "average_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed(),
        ));
        visitor(ExportedStat::float(
            "one_timer",
            "fastest_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.fastest_ball_speed,
        ));
    }
}
