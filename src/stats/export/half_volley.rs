use crate::*;

use super::*;

impl StatFieldProvider for HalfVolleyPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "half_volley",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "half_volley",
            "average_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed(),
        ));
        visitor(ExportedStat::float(
            "half_volley",
            "fastest_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.fastest_ball_speed,
        ));
    }
}

impl StatFieldProvider for HalfVolleyTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "half_volley",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "half_volley",
            "average_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed(),
        ));
        visitor(ExportedStat::float(
            "half_volley",
            "fastest_ball_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.fastest_ball_speed,
        ));
    }
}
