use super::*;
use crate::stats::calculators::{CenterPlayerStats, CenterTeamStats};

impl StatFieldProvider for CenterPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "center",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "center",
            "average_ball_travel_distance",
            StatUnit::UnrealUnits,
            self.average_ball_travel_distance(),
        ));
        visitor(ExportedStat::float(
            "center",
            "average_ball_advance_distance",
            StatUnit::UnrealUnits,
            self.average_ball_advance_distance(),
        ));
        visitor(ExportedStat::float(
            "center",
            "average_lateral_centering_distance",
            StatUnit::UnrealUnits,
            self.average_lateral_centering_distance(),
        ));
    }
}

impl StatFieldProvider for CenterTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "center",
            "count",
            StatUnit::Count,
            self.count,
        ));
        visitor(ExportedStat::float(
            "center",
            "average_ball_travel_distance",
            StatUnit::UnrealUnits,
            self.average_ball_travel_distance(),
        ));
        visitor(ExportedStat::float(
            "center",
            "average_ball_advance_distance",
            StatUnit::UnrealUnits,
            self.average_ball_advance_distance(),
        ));
        visitor(ExportedStat::float(
            "center",
            "average_lateral_centering_distance",
            StatUnit::UnrealUnits,
            self.average_lateral_centering_distance(),
        ));
    }
}
